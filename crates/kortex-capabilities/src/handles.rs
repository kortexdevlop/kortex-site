use crate::{Capability, CapabilityError, policy::Context};
use std::collections::BTreeMap;
pub(crate) fn random_id() -> Result<String, CapabilityError> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|_| CapabilityError::Entropy)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
}
struct Entry {
    owner: Context,
    access: Access,
    expires_ms: u64,
    backend: String,
}
#[derive(Default)]
pub struct Handles {
    entries: BTreeMap<String, Entry>,
}
impl Handles {
    /// Only trusted file picker/storage adapter can create this mapping. Never return backend paths to a guest.
    pub fn issue(
        &mut self,
        owner: Context,
        access: Access,
        backend: String,
        now_ms: u64,
        ttl_ms: u64,
    ) -> Result<String, CapabilityError> {
        if ttl_ms == 0 || ttl_ms > 300000 {
            return Err(CapabilityError::InvalidHandle);
        }
        let id = random_id()?;
        self.entries.insert(
            id.clone(),
            Entry {
                owner,
                access,
                expires_ms: now_ms.saturating_add(ttl_ms),
                backend,
            },
        );
        Ok(id)
    }
    pub fn resolve(
        &self,
        owner: &Context,
        id: &str,
        capability: Capability,
        now: u64,
    ) -> Result<&str, CapabilityError> {
        let entry = self.entries.get(id).ok_or(CapabilityError::InvalidHandle)?;
        if entry.owner != *owner
            || entry.expires_ms <= now
            || !matches!(
                (entry.access, capability),
                (Access::Read, Capability::FileRead) | (Access::Write, Capability::FileSave)
            )
        {
            return Err(CapabilityError::InvalidHandle);
        }
        Ok(&entry.backend)
    }
    pub fn revoke_instance(&mut self, context: &Context) {
        self.entries.retain(|_, entry| entry.owner != *context);
    }
    pub fn revoke(&mut self, id: &str) {
        self.entries.remove(id);
    }
}
