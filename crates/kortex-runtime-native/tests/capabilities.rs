use kortex_capabilities::{
    Capability, CapabilityError,
    policy::{Context, Duration},
    storage::{Owner, StorageBackend, StorageError, StorageRecord, TOOL_QUOTA, ToolStorage},
};
use kortex_runtime_native::{
    files::NativeFiles,
    network::NativeNetwork,
    permissions::{NativeHost, NativeServices},
    storage::NativeStorage,
};
use kortex_sdk::CapabilityRequest;
use serde_json::json;
use std::{
    collections::BTreeMap,
    future::Future,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "kortex-native-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn owner(tool: &str) -> Owner {
    Owner {
        profile: "local".into(),
        tool_id: tool.into(),
    }
}
fn context(tool: &str) -> Context {
    Context {
        tool_id: tool.into(),
        version: "1.0.0".into(),
        instance_id: 1,
    }
}
fn run<T>(f: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::sleep(std::time::Duration::from_millis(1))
    }
}
#[test]
fn durable_isolation_migration_conflict_quota_and_corruption() {
    let temp = Temp::new();
    let mut storage = NativeStorage::new(temp.0.clone()).unwrap();
    let a = owner("tool-a");
    let b = owner("tool-b");
    ToolStorage::new(&mut storage, a.clone(), 1)
        .unwrap()
        .set("prefs/value", b"old".to_vec())
        .unwrap();
    let mut reopened = NativeStorage::new(temp.0.clone()).unwrap();
    assert_eq!(
        ToolStorage::new(&mut reopened, a.clone(), 1)
            .unwrap()
            .get("prefs/value")
            .unwrap(),
        Some(b"old".to_vec())
    );
    assert!(storage.read(&b).unwrap().values.is_empty());
    ToolStorage::new(&mut storage, a.clone(), 2)
        .unwrap()
        .migrate(|mut values| {
            values.insert("next".into(), b"new".to_vec());
            Ok(values)
        })
        .unwrap();
    assert!(
        ToolStorage::new(&mut storage, a.clone(), 1)
            .unwrap()
            .get("prefs/value")
            .unwrap()
            .is_none()
    );
    let before = storage.read(&a).unwrap();
    let stale = StorageRecord {
        revision: before.revision,
        values: BTreeMap::new(),
    };
    assert_eq!(storage.commit(&a, 0, stale), Err(StorageError::Conflict));
    assert_eq!(
        ToolStorage::new(&mut storage, a.clone(), 2)
            .unwrap()
            .set("huge", vec![0; TOOL_QUOTA]),
        Err(StorageError::Quota)
    );
    assert_eq!(storage.read(&a).unwrap(), before);
    assert!(
        ToolStorage::new(&mut storage, a.clone(), 2)
            .unwrap()
            .set("../escape", vec![])
            .is_err()
    );
    assert!(storage.read(&owner("../escape")).is_err());
    assert!(
        ToolStorage::new(&mut storage, a.clone(), 3)
            .unwrap()
            .migrate(|_| Err(StorageError::Corrupt))
            .is_err()
    );
    assert_eq!(storage.read(&a).unwrap(), before);
    let record = std::fs::read_dir(temp.0.join("tool-a"))
        .unwrap()
        .map(|f| f.unwrap().path())
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .unwrap();
    std::fs::write(record, b"corrupt").unwrap();
    assert_eq!(storage.read(&a), Err(StorageError::Corrupt));
}
#[test]
fn files_are_pinned_scoped_bounded_and_cancelled() {
    let temp = Temp::new();
    let path = temp.0.join("selected.txt");
    std::fs::write(&path, b"selected bytes").unwrap();
    let files = NativeFiles::default();
    let a = context("tool-a");
    assert_eq!(
        files.select(&a, None, 0).unwrap(),
        json!({"cancelled":true})
    );
    assert_eq!(
        NativeFiles::save(None, b"x").unwrap(),
        json!({"cancelled":true})
    );
    let selected = files.select(&a, Some(&path), 0).unwrap();
    let token = selected["backend"].as_str().unwrap();
    assert!(selected.get("path").is_none());
    assert_eq!(
        files.read(&context("tool-b"), token, 0, 20, 1),
        Err(CapabilityError::InvalidHandle)
    );
    assert_eq!(
        files.read(&a, token, 0, 262145, 1),
        Err(CapabilityError::SizeLimit)
    );
    assert_eq!(
        files.read(&a, token, 0, 20, 300000),
        Err(CapabilityError::InvalidHandle)
    );
    assert_eq!(
        files.read(&a, token, 0, 20, 1).unwrap()["data_base64"],
        "c2VsZWN0ZWQgYnl0ZXM="
    );
    files.revoke(&a);
    assert_eq!(
        files.read(&a, token, 0, 20, 1),
        Err(CapabilityError::InvalidHandle)
    );
    NativeFiles::save(Some(&path), b"replacement").unwrap();
    assert_eq!(std::fs::read(path).unwrap(), b"replacement");
}
#[test]
fn directory_links_and_traversal_are_refused() {
    let temp = Temp::new();
    let target = temp.0.join("outside");
    let link = temp.0.join("link");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("secret"), b"secret").unwrap();
    #[cfg(windows)]
    {
        assert!(
            std::process::Command::new("node")
                .arg("-e")
                .arg("require('fs').symlinkSync(process.argv[1],process.argv[2],'junction')")
                .arg(&target)
                .arg(&link)
                .status()
                .unwrap()
                .success()
        );
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(NativeStorage::new(link.clone()).is_err());
    assert_eq!(
        NativeFiles::default().select(&context("tool-a"), Some(&link.join("secret")), 0),
        Err(CapabilityError::Denied)
    );
    assert!(NativeFiles::save(Some(&link.join("secret")), b"overwrite").is_err());
    assert_eq!(std::fs::read(target.join("secret")).unwrap(), b"secret");
    #[cfg(windows)]
    std::fs::remove_dir(&link).unwrap();
    #[cfg(unix)]
    std::fs::remove_file(&link).unwrap();
}
#[test]
fn gateway_denies_without_grant_then_persists_and_redacts_audit() {
    let temp = Temp::new();
    let host = NativeHost::new(
        NativeStorage::new(temp.0.clone()).unwrap(),
        NativeNetwork::default(),
    );
    let mut service = NativeServices::new(host);
    let ctx = context("tool-a");
    service.gateway.policy.declare(
        ctx.clone(),
        [
            Capability::StorageWrite,
            Capability::StorageRead,
            Capability::NetworkFetch,
        ],
    );
    let request = CapabilityRequest {
        request_id: "00000000-0000-4000-8000-000000000001".into(),
        capability: "storage.write".into(),
        payload: json!({"key":"prefs","value":"sensitive-test-value"})
            .as_object()
            .unwrap()
            .clone(),
    };
    assert_eq!(
        run(service.fulfill(&ctx, &request, false, Duration::Once))
            .unwrap_err()
            .code,
        "PERMISSION"
    );
    assert_eq!(
        run(service.fulfill(&ctx, &request, true, Duration::Once)).unwrap()["written"],
        true
    );
    let read = CapabilityRequest {
        request_id: "00000000-0000-4000-8000-000000000002".into(),
        capability: "storage.read".into(),
        payload: json!({"key":"prefs"}).as_object().unwrap().clone(),
    };
    assert_eq!(
        run(service.fulfill(&ctx, &read, true, Duration::Once)).unwrap()["value"],
        "sensitive-test-value"
    );
    let network = CapabilityRequest {
        request_id: "00000000-0000-4000-8000-000000000003".into(),
        capability: "network.fetch".into(),
        payload: json!({"url":"https://127.0.0.1/private"})
            .as_object()
            .unwrap()
            .clone(),
    };
    assert_eq!(
        run(service.fulfill(&ctx, &network, true, Duration::Once))
            .unwrap_err()
            .code,
        "NETWORK_DENIED"
    );
    for event in service.gateway.audit.events() {
        assert!(!format!("{event:?}").contains("sensitive-test-value"));
        assert!(!format!("{event:?}").contains("prefs"));
    }
    service.revoke(&ctx);
    assert!(
        !service
            .gateway
            .policy
            .is_declared(&ctx, Capability::StorageRead)
    );
}
#[test]
fn https_only_allowlist_and_unapproved_redirect_targets() {
    for url in [
        "http://example.com/",
        "https://user:pass@example.com/",
        "https://example.com/#fragment",
    ] {
        assert!(NativeNetwork::url(url).is_err());
    }
    assert!(NativeNetwork::new(["https://example.com/path".into()]).is_err());
    let network = NativeNetwork::new(["https://example.com".into()]).unwrap();
    assert!(matches!(
        network.fetch(
            "https://outside.example/",
            &["1.1.1.1".parse().unwrap()],
            std::time::Duration::from_secs(1)
        ),
        Err(CapabilityError::NetworkDenied)
    ));
}

#[test]
#[ignore = "Explicit live HTTPS smoke: cargo test --test capabilities live_https -- --ignored"]
fn live_https_and_external_redirect() {
    let temp = Temp::new();
    let host = NativeHost::new(
        NativeStorage::new(temp.0.clone()).unwrap(),
        NativeNetwork::new(["https://example.com".into(), "https://httpbin.org".into()]).unwrap(),
    );
    let mut service = NativeServices::new(host);
    let ctx = context("network-test");
    service
        .gateway
        .policy
        .declare(ctx.clone(), [Capability::NetworkFetch]);
    let mut request = CapabilityRequest {
        request_id: "00000000-0000-4000-8000-000000000010".into(),
        capability: "network.fetch".into(),
        payload: json!({"url":"https://example.com/"})
            .as_object()
            .unwrap()
            .clone(),
    };
    assert_eq!(
        run(service.fulfill(&ctx, &request, true, Duration::Once)).unwrap()["status"],
        200
    );
    request.request_id = "00000000-0000-4000-8000-000000000011".into();
    request.payload =
        json!({"url":"https://httpbin.org/redirect-to?url=https%3A%2F%2Fexample.com%2F"})
            .as_object()
            .unwrap()
            .clone();
    assert_eq!(
        run(service.fulfill(&ctx, &request, true, Duration::Once))
            .unwrap_err()
            .code,
        "NETWORK_DENIED"
    );
}

#[test]
fn existing_session_permission_is_reused_and_once_is_consumed() {
    let temp = Temp::new();
    let host = NativeHost::new(
        NativeStorage::new(temp.0.clone()).unwrap(),
        NativeNetwork::default(),
    );
    let mut service = NativeServices::new(host);
    let ctx = context("tool-a");
    service
        .gateway
        .policy
        .declare(ctx.clone(), [Capability::StorageRead]);
    let request = CapabilityRequest {
        request_id: "00000000-0000-4000-8000-000000000020".into(),
        capability: "storage.read".into(),
        payload: json!({"key":"prefs"}).as_object().unwrap().clone(),
    };
    assert_eq!(
        run(service.execute_existing(&ctx, &request))
            .unwrap_err()
            .code,
        "PERMISSION"
    );
    run(service.fulfill(&ctx, &request, true, Duration::Once)).unwrap();
    assert_eq!(
        run(service.execute_existing(&ctx, &request))
            .unwrap_err()
            .code,
        "PERMISSION"
    );
    run(service.fulfill(&ctx, &request, true, Duration::Session)).unwrap();
    assert_eq!(
        run(service.execute_existing(&ctx, &request)).unwrap()["value"],
        serde_json::Value::Null
    );
    service.revoke(&ctx);
    assert_eq!(
        run(service.execute_existing(&ctx, &request))
            .unwrap_err()
            .code,
        "PERMISSION"
    );
}

#[test]
fn trusted_dialog_can_complete_after_the_start_gesture_expires() {
    use kortex_capabilities::{
        AuthorizedCall, CapabilityCall, CapabilityHost, CapabilityResponse, Gateway, Resource,
    };
    struct Dialog(std::sync::Arc<AtomicU64>);
    impl CapabilityHost for Dialog {
        async fn execute(
            &mut self,
            call: &AuthorizedCall<'_>,
        ) -> Result<CapabilityResponse, CapabilityError> {
            assert_eq!(call.expires_ms(), 5000);
            self.0.store(1500, Ordering::SeqCst);
            Ok(CapabilityResponse::Bytes(vec![]))
        }
    }
    let clock = std::sync::Arc::new(AtomicU64::new(0));
    let mut gateway = Gateway::new(Dialog(clock.clone()));
    let ctx = context("tool-a");
    gateway.policy.declare(ctx.clone(), [Capability::FilePick]);
    gateway
        .policy
        .grant(
            &ctx,
            Capability::FilePick,
            &Resource::FileDialog,
            Duration::Once,
            5000,
        )
        .unwrap();
    let gesture = gateway.policy.record_gesture(&ctx, 0).unwrap();
    let request = CapabilityCall {
        request_id: "00000000-0000-4000-8000-000000000030".into(),
        capability: Capability::FilePick,
        resource: Resource::FileDialog,
        bytes: vec![],
        max_output_bytes: 10,
    };
    assert!(
        run(gateway.execute(&ctx, request, Some(&gesture), &|| clock
            .load(Ordering::SeqCst)))
        .is_ok()
    );
}
