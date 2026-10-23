//! Versioned non-secret state snapshots. Checksums detect corruption, not authenticity.
use kortex_capabilities::storage::{StorageError, TOOL_QUOTA};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub tool_id: String,
    pub tool_version: String,
    pub storage_version: u32,
    pub state: Value,
}
impl Snapshot {
    pub fn encode(&self) -> Result<Vec<u8>, StorageError> {
        if self.tool_id.is_empty() || self.tool_version.is_empty() || self.storage_version == 0 {
            return Err(StorageError::Corrupt);
        }
        let payload = json!({"tool_id":self.tool_id,"tool_version":self.tool_version,"storage_version":self.storage_version,"state":self.state});
        let checksum = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&payload).map_err(|_| StorageError::Corrupt)?)
        );
        let bytes = serde_json::to_vec(&json!({"format":1,"payload":payload,"sha256":checksum}))
            .map_err(|_| StorageError::Corrupt)?;
        if bytes.len() > TOOL_QUOTA {
            return Err(StorageError::Quota);
        }
        Ok(bytes)
    }
    /// Restore only to the selected tool and storage version. N-1 requires an explicit migration.
    pub fn restore(bytes: &[u8], tool_id: &str, version: u32) -> Result<Self, StorageError> {
        if bytes.len() > TOOL_QUOTA {
            return Err(StorageError::Quota);
        }
        let root: Value = serde_json::from_slice(bytes).map_err(|_| StorageError::Corrupt)?;
        let object = root.as_object().ok_or(StorageError::Corrupt)?;
        if object.len() != 3 || root["format"] != 1 {
            return Err(StorageError::Corrupt);
        }
        let p = root
            .get("payload")
            .and_then(Value::as_object)
            .ok_or(StorageError::Corrupt)?;
        if p.len() != 4 || !p.contains_key("state") {
            return Err(StorageError::Corrupt);
        }
        let checksum = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(p).map_err(|_| StorageError::Corrupt)?)
        );
        if root["sha256"].as_str() != Some(&checksum) {
            return Err(StorageError::Corrupt);
        }
        let id = p
            .get("tool_id")
            .and_then(Value::as_str)
            .ok_or(StorageError::Corrupt)?;
        let v = p
            .get("storage_version")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .filter(|v| *v > 0)
            .ok_or(StorageError::Corrupt)?;
        let tool_version = p
            .get("tool_version")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or(StorageError::Corrupt)?;
        if id != tool_id || id.is_empty() || v != version {
            return Err(StorageError::Permission);
        }
        Ok(Self {
            tool_id: id.into(),
            tool_version: tool_version.into(),
            storage_version: v,
            state: p["state"].clone(),
        })
    }
}
