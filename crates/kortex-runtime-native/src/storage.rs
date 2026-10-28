//! Host-selected, bounded records. Keys never become filesystem paths.
use kortex_capabilities::storage::{
    Owner, StorageBackend, StorageError, StorageRecord, validate_record,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    format: u8,
    revision: u64,
    values: Vec<(u32, String, Vec<u8>)>,
    checksum: String,
}
pub(crate) fn check_path(path: &Path) -> std::io::Result<()> {
    for part in path.ancestors() {
        match fs::symlink_metadata(part) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
            Ok(m) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if m.file_attributes() & 0x400 != 0 {
                        return Err(std::io::ErrorKind::PermissionDenied.into());
                    }
                }
                if m.file_type().is_symlink() {
                    return Err(std::io::ErrorKind::PermissionDenied.into());
                }
            }
        }
    }
    Ok(())
}
pub(crate) fn open(path: &Path, write: bool, create: bool) -> std::io::Result<File> {
    check_path(path)?;
    let mut options = OpenOptions::new();
    options.read(true).write(write).create(create);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000);
    }
    let file = options.open(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if file.metadata()?.file_attributes() & 0x400 != 0 {
            return Err(std::io::ErrorKind::PermissionDenied.into());
        }
    }
    if !file.metadata()?.is_file() {
        return Err(std::io::ErrorKind::PermissionDenied.into());
    }
    Ok(file)
}
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    check_path(path)?;
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|_| std::io::Error::other("entropy unavailable"))?;
    let temp = path.with_file_name(format!(".kortex-{:x}.tmp", u128::from_le_bytes(random)));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = (|| {
        f.write_all(bytes)?;
        f.sync_all()?;
        check_path(path)?;
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
pub struct NativeStorage {
    root: PathBuf,
    cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}
impl NativeStorage {
    pub fn app_data() -> Result<Self, StorageError> {
        Self::new(
            dirs::data_local_dir()
                .ok_or(StorageError::Unavailable)?
                .join("kortex")
                .join("tools"),
        )
    }
    /// Trusted host only; tests use their own temporary directory.
    pub fn new(root: PathBuf) -> Result<Self, StorageError> {
        check_path(&root).map_err(|_| StorageError::Permission)?;
        fs::create_dir_all(&root).map_err(|_| StorageError::Unavailable)?;
        Ok(Self {
            root: root.canonicalize().map_err(|_| StorageError::Unavailable)?,
            cancel: None,
        })
    }
    pub(crate) fn cancellation(&mut self, cancel: std::sync::Arc<std::sync::atomic::AtomicBool>) {
        self.cancel = Some(cancel)
    }
    fn check_cancel(&self) -> Result<(), StorageError> {
        if self
            .cancel
            .as_ref()
            .is_some_and(|v| v.load(std::sync::atomic::Ordering::SeqCst))
        {
            Err(StorageError::Cancelled)
        } else {
            Ok(())
        }
    }
    fn dir(&self, owner: &Owner) -> Result<PathBuf, StorageError> {
        if owner.tool_id.len() < 3
            || owner.tool_id.len() > 64
            || !owner
                .tool_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || owner.profile.is_empty()
            || owner.profile.len() > 256
        {
            return Err(StorageError::Permission);
        }
        let dir = self.root.join(&owner.tool_id);
        check_path(&dir).map_err(|_| StorageError::Permission)?;
        fs::create_dir_all(&dir).map_err(|_| StorageError::Unavailable)?;
        Ok(dir)
    }
    fn path(&self, owner: &Owner) -> Result<PathBuf, StorageError> {
        Ok(self.dir(owner)?.join(format!(
            "{:x}.json",
            Sha256::digest(owner.profile.as_bytes())
        )))
    }
    /// Trusted host integration: the exact per-profile file; never exposed to a guest.
    pub fn profile_file(&self, owner:&Owner)->Result<PathBuf,StorageError>{self.path(owner)}
    fn read_record(path: &Path) -> Result<StorageRecord, StorageError> {
        let f = match open(path, false, false) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(StorageRecord::default());
            }
            Err(_) => return Err(StorageError::Permission),
        };
        let mut bytes = Vec::new();
        f.take(24 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| StorageError::Unavailable)?;
        if bytes.len() > 24 * 1024 * 1024 {
            return Err(StorageError::Quota);
        }
        let r: Record = serde_json::from_slice(&bytes).map_err(|_| StorageError::Corrupt)?;
        if r.format != 1
            || format!(
                "{:x}",
                Sha256::digest(
                    serde_json::to_vec(&(r.revision, &r.values))
                        .map_err(|_| StorageError::Corrupt)?
                )
            ) != r.checksum
        {
            return Err(StorageError::Corrupt);
        }
        let count = r.values.len();
        let record = StorageRecord {
            revision: r.revision,
            values: r.values.into_iter().map(|(v, k, b)| ((v, k), b)).collect(),
        };
        if count != record.values.len() {
            return Err(StorageError::Corrupt);
        }
        validate_record(&record)?;
        Ok(record)
    }
}
impl StorageBackend for NativeStorage {
    fn read(&self, owner: &Owner) -> Result<StorageRecord, StorageError> {
        Self::read_record(&self.path(owner)?)
    }
    fn commit(
        &mut self,
        owner: &Owner,
        expected: u64,
        next: StorageRecord,
    ) -> Result<(), StorageError> {
        validate_record(&next)?;
        if expected.checked_add(1) != Some(next.revision) {
            return Err(StorageError::Conflict);
        }
        self.check_cancel()?;
        let path = self.path(owner)?;
        let lock =
            open(&path.with_extension("lock"), true, true).map_err(|_| StorageError::Permission)?;
        let started = std::time::Instant::now();
        loop {
            self.check_cancel()?;
            match fs2::FileExt::try_lock_exclusive(&lock) {
                Ok(()) => break,
                Err(_) if started.elapsed() < std::time::Duration::from_secs(3) => {
                    std::thread::sleep(std::time::Duration::from_millis(10))
                }
                Err(_) => return Err(StorageError::Conflict),
            }
        }
        if Self::read_record(&path)?.revision != expected {
            return Err(StorageError::Conflict);
        }
        let values = next
            .values
            .into_iter()
            .map(|((v, k), b)| (v, k, b))
            .collect::<Vec<_>>();
        let checksum = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(next.revision, &values)).map_err(|_| StorageError::Corrupt)?
            )
        );
        let bytes = serde_json::to_vec(&Record {
            format: 1,
            revision: next.revision,
            values,
            checksum,
        })
        .map_err(|_| StorageError::Corrupt)?;
        if bytes.len() > 24 * 1024 * 1024 {
            return Err(StorageError::Quota);
        }
        self.check_cancel()?;
        atomic_write(&path, &bytes).map_err(|_| StorageError::Unavailable)
    }
}
