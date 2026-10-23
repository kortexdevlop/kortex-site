//! Test-only injected engine. This file is explicitly registered as an integration test.
use ed25519_dalek::{Signer, SigningKey};
use kortex_abi::{Envelope, Reply, RequestKind};
use kortex_core::{dispatcher::Clock, *};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::BTreeMap,
    future::Future,
    sync::atomic::{AtomicIsize, Ordering},
};
struct Tracked;
static LIVE: AtomicIsize = AtomicIsize::new(0);
unsafe impl GlobalAlloc for Tracked {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as isize, Ordering::Relaxed);
        unsafe { System.dealloc(p, l) }
    }
}
#[global_allocator]
static ALLOC: Tracked = Tracked;
fn run<T>(f: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    match f.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(v) => v,
        _ => panic!("test future pending"),
    }
}
struct TestClock;
impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        0
    }
    async fn wait_until(&self, _: u64) {
        std::future::pending::<()>().await
    }
}
struct RecordedEngine {
    records: Vec<Value>,
}
impl GuestEngine for RecordedEngine {
    type Handle = ();
    async fn load(&mut self, _: &VerifiedTool) -> Result<(), RuntimeError> {
        Ok(())
    }
    async fn dispatch(&mut self, _: &mut (), r: &Envelope, _: u64) -> Result<Reply, RuntimeError> {
        let encoded = r.encode().unwrap();
        assert_eq!(Envelope::decode(&encoded).unwrap(), *r);
        let name = serde_json::to_value(&r.kind).unwrap();
        let record = self.records.iter().find(|v| v["kind"] == name).unwrap();
        let data = if r.kind == RequestKind::ActionInvoke {
            json!({"result":r.payload["input"]["a"].as_f64().unwrap()+r.payload["input"]["b"].as_f64().unwrap()})
        } else if r.kind == RequestKind::UiEvent {
            assert_eq!(Value::Object(r.payload.clone()), record["payload"]);
            record["data"].clone()
        } else {
            record["data"].clone()
        };
        Ok(Reply {
            v: 1,
            request_id: r.request_id.clone(),
            ok: true,
            data: Some(data.as_object().unwrap().clone()),
            error: None,
        })
    }
    async fn unload(&mut self, _: ()) -> Result<(), RuntimeError> {
        Ok(())
    }
}
fn tool() -> VerifiedTool {
    let m: Value =
        serde_json::from_str(include_str!("../../shared/fixtures/manifest.valid.json")).unwrap();
    let c: Value =
        serde_json::from_str(include_str!("../../shared/fixtures/contract.valid.json")).unwrap();
    let bytes = b"signed test-only recorded engine input".to_vec();
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let key = SigningKey::from_bytes(&[9; 32]);
    let text = format!("kortex-artifact-v1\ndemo-calculator\n1.0.0\n1.0\n{sha256}\n");
    let p = ArtifactDescriptor {
        schema_version: 1,
        tool_id: "demo-calculator".into(),
        version: "1.0.0".into(),
        abi_version: "1.0".into(),
        sha256,
        signing_key_id: "test-only".into(),
        signature: key.sign(text.as_bytes()).to_bytes(),
    };
    VerifiedTool::verify(
        &m,
        &c,
        bytes,
        &p,
        &BTreeMap::from([("test-only".into(), key.verifying_key().to_bytes())]),
    )
    .unwrap()
}
#[test]
fn recorded_scenario_and_one_hundred_actions_have_bounded_retained_memory() {
    let records: Vec<Value> = include_str!("../fixtures/events.ndjson")
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let event = records[2]["payload"].clone();
    let mut rt = Runtime::new(RecordedEngine { records }, TestClock);
    let id = run(rt.load(tool())).unwrap();
    let ui = run(rt.initialize(id)).unwrap().data.unwrap();
    assert_eq!(ui["key"], "status");
    assert!(run(rt.dispatch_event(id, event)).unwrap().ok);
    assert!(
        run(rt.dispatch_capability_result(
            id,
            "00000000-0000-4000-8000-000000000001".into(),
            Ok(Default::default())
        ))
        .unwrap()
        .ok
    );
    for _ in 0..10 {
        run(rt.invoke_action(id, "add", json!({"a":2,"b":3}))).unwrap();
    }
    let baseline = LIVE.load(Ordering::Relaxed);
    for i in 0..100 {
        let reply = run(rt.invoke_action(id, "add", json!({"a":i,"b":3}))).unwrap();
        assert_eq!(reply.data.unwrap()["result"], json!(i as f64 + 3.0));
        assert_eq!(rt.instance(id).unwrap().queued(), 0);
        assert_eq!(rt.instance_count(), 1);
    }
    let retained = LIVE.load(Ordering::Relaxed) - baseline;
    assert!(retained <= 16384, "retained allocation growth {retained}");
    for input in [
        json!({}),
        json!({"a":"bad","b":2}),
        json!({"a":1,"b":2,"extra":3}),
    ] {
        assert_eq!(
            run(rt.invoke_action(id, "add", input)),
            Err(RuntimeError::InvalidInput)
        );
    }
    run(rt.shutdown(id)).unwrap();
    assert_eq!(rt.instance_count(), 0);
    println!(
        "100 actions: retained allocation delta {retained} bytes, queues drained, registry empty after shutdown"
    );
}
