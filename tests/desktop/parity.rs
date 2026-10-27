#[path = "../../crates/kortex-runtime-native/tests/common/mod.rs"]
mod common;
use common::{run, tool_with};
use kortex_capabilities::{
    policy::Duration,
    storage::{Owner, ToolStorage},
};
use kortex_core::{Runtime, dispatcher::Clock};
use kortex_runtime_native::{
    HostedNativeEngine,
    network::NativeNetwork,
    permissions::{CapabilityInbox, NativeHost, NativeServices, now_ms},
    storage::NativeStorage,
};
use serde_json::json;
struct TestClock;
impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        now_ms()
    }
    async fn wait_until(&self, d: u64) {
        std::future::poll_fn(|cx| {
            if now_ms() >= d {
                std::task::Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
        })
        .await
    }
}
#[test]
fn identical_sdk_guest_consent_storage_ui_and_reopen() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let wasm = std::fs::read(root.join(
        "target/web-test-guest/wasm32-unknown-unknown/release/kortex_browser_capability_test.wasm",
    ))
    .expect("run web-adapter/prepare-tests.mjs first");
    let tool = tool_with(&wasm, &["storage.read"]);
    let inbox = CapabilityInbox::default();
    let mut runtime = Runtime::new(HostedNativeEngine::new(inbox.clone()), TestClock);
    let dir =
        std::env::temp_dir().join(format!("kortex-parity-{}-{}", std::process::id(), now_ms()));
    let mut storage = NativeStorage::new(dir.clone()).unwrap();
    ToolStorage::new(
        &mut storage,
        Owner {
            profile: "local".into(),
            tool_id: "demo-calculator".into(),
        },
        1,
    )
    .unwrap()
    .set("prefs", b"123".to_vec())
    .unwrap();
    let mut services = NativeServices::new(NativeHost::new(storage, NativeNetwork::default()));
    for allow in [false, true] {
        let id = run(runtime.load(tool.clone())).unwrap();
        let context = runtime
            .prepare_capabilities(id, &mut services.gateway)
            .unwrap();
        let initial = run(runtime.initialize(id)).unwrap();
        assert_eq!(initial.data.unwrap()["props"]["text"], "pending");
        let request = inbox.pop().unwrap();
        let result = run(services.fulfill(&context, &request, allow, Duration::Once));
        run(runtime.dispatch_capability_result(id, request.request_id, result)).unwrap();
        let rendered = run(runtime.render(id)).unwrap();
        let expected = if allow { "resolved:123" } else { "PERMISSION" };
        assert_eq!(
            rendered.data.unwrap(),
            json!({"type":"text","key":"outcome","props":{"text":expected},"children":[]})
                .as_object()
                .unwrap()
                .clone()
        );
        run(runtime.shutdown_with_capabilities(id, &mut services.gateway)).unwrap();
        services.revoke(&context);
        assert_eq!(runtime.instance_count(), 0);
        assert!(inbox.pop().is_none());
    }
    std::fs::remove_dir_all(dir).unwrap();
}
