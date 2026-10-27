use base64::Engine;
use kortex_capabilities::{CapabilityError, policy::Context};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
    sync::{Arc, Mutex},
};
struct Entry {
    owner: Context,
    file: File,
    expires: u64,
}
#[derive(Clone, Default)]
pub struct NativeFiles {
    entries: Arc<Mutex<BTreeMap<String, Entry>>>,
}
impl NativeFiles {
    pub fn select(
        &self,
        owner: &Context,
        path: Option<&Path>,
        now: u64,
    ) -> Result<Value, CapabilityError> {
        let Some(path) = path else {
            return Ok(json!({"cancelled":true}));
        };
        let file = crate::storage::open(path, false, false).map_err(|_| CapabilityError::Denied)?;
        let size = file
            .metadata()
            .map_err(|_| CapabilityError::HostFailure)?
            .len();
        if size > 10 * 1024 * 1024 {
            return Err(CapabilityError::SizeLimit);
        }
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CapabilityError::HostFailure)?;
        entries.retain(|_, e| e.expires > now);
        if entries.len() >= 128 {
            return Err(CapabilityError::SizeLimit);
        }
        let mut entropy = [0; 32];
        getrandom::fill(&mut entropy).map_err(|_| CapabilityError::Entropy)?;
        let token = entropy
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        entries.insert(
            token.clone(),
            Entry {
                owner: owner.clone(),
                file,
                expires: now.saturating_add(300000),
            },
        );
        Ok(
            json!({"backend":token,"name":path.file_name().and_then(|s|s.to_str()).unwrap_or("file"),"size":size}),
        )
    }
    pub fn read(
        &self,
        owner: &Context,
        backend: &str,
        offset: u64,
        length: usize,
        now: u64,
    ) -> Result<Value, CapabilityError> {
        if length == 0 || length > 256 * 1024 {
            return Err(CapabilityError::SizeLimit);
        }
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| CapabilityError::HostFailure)?;
        let e = entries
            .get_mut(backend)
            .ok_or(CapabilityError::InvalidHandle)?;
        if &e.owner != owner || e.expires <= now {
            return Err(CapabilityError::InvalidHandle);
        }
        let size = e
            .file
            .metadata()
            .map_err(|_| CapabilityError::HostFailure)?
            .len();
        if size > 10 * 1024 * 1024 || offset > size {
            return Err(CapabilityError::SizeLimit);
        }
        e.file
            .seek(SeekFrom::Start(offset))
            .map_err(|_| CapabilityError::HostFailure)?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut e.file)
            .take(length as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| CapabilityError::HostFailure)?;
        Ok(
            json!({"data_base64":base64::engine::general_purpose::STANDARD.encode(&bytes),"offset":offset,"eof":offset+bytes.len() as u64>=size}),
        )
    }
    pub fn save(path: Option<&Path>, bytes: &[u8]) -> Result<Value, CapabilityError> {
        if bytes.len() > 10 * 1024 * 1024 {
            return Err(CapabilityError::SizeLimit);
        }
        let Some(path) = path else {
            return Ok(json!({"cancelled":true}));
        };
        crate::storage::atomic_write(path, bytes).map_err(|_| CapabilityError::Denied)?;
        Ok(json!({"saved":true}))
    }
    pub fn revoke(&self, owner: &Context) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|_, e| &e.owner != owner)
        }
    }
}
