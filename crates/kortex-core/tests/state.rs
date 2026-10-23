use kortex_capabilities::storage::*;
use kortex_core::state::Snapshot;
use serde_json::json;
use std::collections::BTreeMap;
#[derive(Default)]
struct Memory {
    records: BTreeMap<Owner, StorageRecord>,
    fail: Option<StorageError>,
}
impl StorageBackend for Memory {
    fn read(&self, owner: &Owner) -> Result<StorageRecord, StorageError> {
        if let Some(e) = self.fail {
            return Err(e);
        }
        Ok(self.records.get(owner).cloned().unwrap_or_default())
    }
    fn commit(
        &mut self,
        owner: &Owner,
        expected: u64,
        next: StorageRecord,
    ) -> Result<(), StorageError> {
        if let Some(e) = self.fail {
            return Err(e);
        }
        if self.read(owner)?.revision != expected {
            return Err(StorageError::Conflict);
        }
        validate_record(&next)?;
        self.records.insert(owner.clone(), next);
        Ok(())
    }
}
fn owner(profile: &str, tool: &str) -> Owner {
    Owner {
        profile: profile.into(),
        tool_id: tool.into(),
    }
}
#[test]
fn roundtrip_and_corruption() {
    let snapshot = Snapshot {
        tool_id: "test.tool".into(),
        tool_version: "1.0.0".into(),
        storage_version: 1,
        state: json!({"count":42}),
    };
    let bytes = snapshot.encode().unwrap();
    assert_eq!(Snapshot::restore(&bytes, "test.tool", 1).unwrap(), snapshot);
    assert_eq!(
        Snapshot::restore(&bytes, "other", 1),
        Err(StorageError::Permission)
    );
    assert_eq!(
        Snapshot::restore(&bytes, "test.tool", 2),
        Err(StorageError::Permission)
    );
    let mut altered: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    altered["payload"]["state"] = json!(false);
    assert_eq!(
        Snapshot::restore(&serde_json::to_vec(&altered).unwrap(), "test.tool", 1),
        Err(StorageError::Corrupt)
    );
    assert_eq!(
        Snapshot::restore(b"broken", "test.tool", 1),
        Err(StorageError::Corrupt)
    );
}
#[test]
fn isolation_cleanup_and_export() {
    let mut memory = Memory::default();
    for (p, t) in [("a", "one"), ("b", "one"), ("a", "two")] {
        ToolStorage::new(&mut memory, owner(p, t), 1)
            .unwrap()
            .set("value", format!("{p}:{t}").into_bytes())
            .unwrap();
    }
    let mut storage = ToolStorage::new(&mut memory, owner("a", "one"), 1).unwrap();
    assert_eq!(
        storage.export_user_approved(false),
        Err(StorageError::Permission)
    );
    assert_eq!(storage.export_user_approved(true).unwrap().len(), 1);
    assert_eq!(storage.get("../other"), Err(StorageError::Corrupt));
    storage.delete("value").unwrap();
    assert_eq!(storage.get("value").unwrap(), None);
    storage.set("new", vec![1]).unwrap();
    storage.delete_namespace().unwrap();
    assert!(storage.export_user_approved(true).unwrap().is_empty());
    for (p, t) in [("b", "one"), ("a", "two")] {
        assert_eq!(
            ToolStorage::new(&mut memory, owner(p, t), 1)
                .unwrap()
                .get("value")
                .unwrap(),
            Some(format!("{p}:{t}").into_bytes())
        );
    }
}
#[test]
fn quota_is_atomic_and_shared_across_versions() {
    let mut memory = Memory::default();
    let identity = owner("a", "one");
    ToolStorage::new(&mut memory, identity.clone(), 1)
        .unwrap()
        .set("x", vec![0; TOOL_QUOTA - 1])
        .unwrap();
    let before = memory.read(&identity).unwrap();
    assert_eq!(
        ToolStorage::new(&mut memory, identity.clone(), 2)
            .unwrap()
            .set("y", vec![1]),
        Err(StorageError::Quota)
    );
    assert_eq!(memory.read(&identity).unwrap(), before);
    ToolStorage::new(&mut memory, identity.clone(), 2)
        .unwrap()
        .migrate(Ok)
        .unwrap();
    assert_eq!(
        ToolStorage::new(&mut memory, identity, 2)
            .unwrap()
            .get("x")
            .unwrap()
            .unwrap()
            .len(),
        TOOL_QUOTA - 1
    );
}
#[test]
fn migration_failure_rolls_back() {
    let mut memory = Memory::default();
    let identity = owner("a", "one");
    ToolStorage::new(&mut memory, identity.clone(), 1)
        .unwrap()
        .set("count", vec![1])
        .unwrap();
    let before = memory.read(&identity).unwrap();
    assert_eq!(
        ToolStorage::new(&mut memory, identity.clone(), 2)
            .unwrap()
            .migrate(|_| Err(StorageError::Cancelled)),
        Err(StorageError::Cancelled)
    );
    assert_eq!(memory.read(&identity).unwrap(), before);
    ToolStorage::new(&mut memory, identity.clone(), 2)
        .unwrap()
        .migrate(|mut values| {
            values.insert("count".into(), vec![2]);
            Ok(values)
        })
        .unwrap();
    assert_eq!(
        ToolStorage::new(&mut memory, identity.clone(), 1)
            .unwrap()
            .get("count")
            .unwrap(),
        None
    );
    assert_eq!(
        ToolStorage::new(&mut memory, identity, 2)
            .unwrap()
            .get("count")
            .unwrap(),
        Some(vec![2])
    );
}
#[test]
fn recoverable_backend_errors_and_conflict() {
    for error in [
        StorageError::Permission,
        StorageError::Unavailable,
        StorageError::Cancelled,
    ] {
        let mut memory = Memory {
            fail: Some(error),
            ..Default::default()
        };
        assert_eq!(
            ToolStorage::new(&mut memory, owner("a", "one"), 1)
                .unwrap()
                .set("x", vec![1]),
            Err(error)
        );
        assert!(memory.records.is_empty());
        memory.fail = None;
        ToolStorage::new(&mut memory, owner("a", "one"), 1)
            .unwrap()
            .set("x", vec![1])
            .unwrap();
    }
    let mut memory = Memory::default();
    let identity = owner("a", "one");
    ToolStorage::new(&mut memory, identity.clone(), 1)
        .unwrap()
        .set("x", vec![1])
        .unwrap();
    assert_eq!(
        memory.commit(&identity, 0, StorageRecord::default()),
        Err(StorageError::Conflict)
    );
    assert_eq!(memory.read(&identity).unwrap().revision, 1);
}

#[test]
fn invalid_migration_and_oversized_snapshot_are_rejected() {
    let mut memory = Memory::default();
    let identity = owner("a", "one");
    ToolStorage::new(&mut memory, identity.clone(), 1)
        .unwrap()
        .set("x", vec![1])
        .unwrap();
    let before = memory.read(&identity).unwrap();
    assert_eq!(
        ToolStorage::new(&mut memory, identity.clone(), 2)
            .unwrap()
            .migrate(|_| Ok(BTreeMap::from([("../escape".into(), vec![1])]))),
        Err(StorageError::Corrupt)
    );
    assert_eq!(memory.read(&identity).unwrap(), before);
    assert_eq!(
        ToolStorage::new(&mut memory, identity.clone(), 2)
            .unwrap()
            .migrate(|_| Ok(BTreeMap::from([("x".into(), vec![0; TOOL_QUOTA])]))),
        Err(StorageError::Quota)
    );
    assert_eq!(memory.read(&identity).unwrap(), before);
    let snapshot = Snapshot {
        tool_id: "one".into(),
        tool_version: "1.0.0".into(),
        storage_version: 1,
        state: json!("x".repeat(TOOL_QUOTA)),
    };
    assert_eq!(snapshot.encode(), Err(StorageError::Quota));
}
