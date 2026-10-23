use std::collections::BTreeMap;

pub const TOOL_QUOTA: usize = 5 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    Quota,
    Permission,
    Unavailable,
    Cancelled,
    Conflict,
    Corrupt,
}
impl StorageError {
    pub fn code(self) -> &'static str {
        match self {
            Self::Quota => "QUOTA",
            Self::Permission => "PERMISSION",
            Self::Unavailable => "UNAVAILABLE",
            Self::Cancelled => "CANCELLED",
            Self::Conflict => "CONFLICT",
            Self::Corrupt => "CORRUPT",
        }
    }
}
/// Host-selected identity. Never construct from guest-supplied profile/tool identifiers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Owner {
    pub profile: String,
    pub tool_id: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageRecord {
    pub revision: u64,
    pub values: BTreeMap<(u32, String), Vec<u8>>,
}
/// Backends must atomically compare revision and replace the entire owner record.
/// Failure must leave it unchanged. Missing records have revision zero. Implementations
/// must isolate owners, enforce TOOL_QUOTA including key bytes across all versions,
/// and return recoverable errors. No durable backend is provided by this module.
pub trait StorageBackend {
    fn read(&self, owner: &Owner) -> Result<StorageRecord, StorageError>;
    fn commit(
        &mut self,
        owner: &Owner,
        expected_revision: u64,
        next: StorageRecord,
    ) -> Result<(), StorageError>;
}
/// Scoped host service; access policy is checked by the capability gateway before use.
/// Only non-secret tool state is allowed. Credentials belong to host secret services.
pub struct ToolStorage<'a, B> {
    backend: &'a mut B,
    owner: Owner,
    version: u32,
}
impl<'a, B: StorageBackend> ToolStorage<'a, B> {
    pub fn new(backend: &'a mut B, owner: Owner, version: u32) -> Result<Self, StorageError> {
        if owner.profile.is_empty() || owner.tool_id.is_empty() || version == 0 {
            return Err(StorageError::Permission);
        }
        Ok(Self {
            backend,
            owner,
            version,
        })
    }
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        valid_key(key)?;
        Ok(self
            .backend
            .read(&self.owner)?
            .values
            .get(&(self.version, key.into()))
            .cloned())
    }
    pub fn set(&mut self, key: &str, value: Vec<u8>) -> Result<(), StorageError> {
        valid_key(key)?;
        let version = self.version;
        self.change(|r| {
            r.values.insert((version, key.into()), value);
            Ok(())
        })
    }
    pub fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        valid_key(key)?;
        let version = self.version;
        self.change(|r| {
            r.values.remove(&(version, key.into()));
            Ok(())
        })
    }
    pub fn delete_namespace(&mut self) -> Result<(), StorageError> {
        let version = self.version;
        self.change(|r| {
            r.values.retain(|(v, _), _| *v != version);
            Ok(())
        })
    }
    /// Explicit N-1 migration, atomically replacing the previous namespace.
    pub fn migrate(
        &mut self,
        transform: impl FnOnce(
            BTreeMap<String, Vec<u8>>,
        ) -> Result<BTreeMap<String, Vec<u8>>, StorageError>,
    ) -> Result<(), StorageError> {
        let previous = self
            .version
            .checked_sub(1)
            .filter(|v| *v > 0)
            .ok_or(StorageError::Corrupt)?;
        let version = self.version;
        self.change(|r| {
            if r.values.keys().any(|(v, _)| *v == version) {
                return Err(StorageError::Conflict);
            }
            let old = r
                .values
                .iter()
                .filter(|((v, _), _)| *v == previous)
                .map(|((_, k), v)| (k.clone(), v.clone()))
                .collect();
            let migrated = transform(old)?;
            for key in migrated.keys() {
                valid_key(key)?;
            }
            r.values.retain(|(v, _), _| *v != previous);
            r.values
                .extend(migrated.into_iter().map(|(k, v)| ((version, k), v)));
            Ok(())
        })
    }
    /// Host UI calls this only after explicit user approval; never expose as a guest capability.
    pub fn export_user_approved(
        &self,
        approved: bool,
    ) -> Result<BTreeMap<String, Vec<u8>>, StorageError> {
        if !approved {
            return Err(StorageError::Permission);
        }
        Ok(self
            .backend
            .read(&self.owner)?
            .values
            .into_iter()
            .filter(|((v, _), _)| *v == self.version)
            .map(|((_, k), v)| (k, v))
            .collect())
    }
    fn change(
        &mut self,
        edit: impl FnOnce(&mut StorageRecord) -> Result<(), StorageError>,
    ) -> Result<(), StorageError> {
        let mut record = self.backend.read(&self.owner)?;
        let revision = record.revision;
        edit(&mut record)?;
        validate_record(&record)?;
        record.revision = revision.checked_add(1).ok_or(StorageError::Unavailable)?;
        self.backend.commit(&self.owner, revision, record)
    }
}
pub fn validate_record(record: &StorageRecord) -> Result<(), StorageError> {
    let mut size = 0usize;
    for ((version, key), value) in &record.values {
        if *version == 0 {
            return Err(StorageError::Corrupt);
        }
        valid_key(key)?;
        size = size
            .checked_add(key.len())
            .and_then(|n| n.checked_add(value.len()))
            .ok_or(StorageError::Quota)?;
        if size > TOOL_QUOTA {
            return Err(StorageError::Quota);
        }
    }
    Ok(())
}
fn valid_key(key: &str) -> Result<(), StorageError> {
    if key.is_empty()
        || key.len() > 256
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
        || key
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
    {
        Err(StorageError::Corrupt)
    } else {
        Ok(())
    }
}
