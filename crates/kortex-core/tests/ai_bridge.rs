// Explicit test-only engine; production execution is verified separately with signed WASM.
use ed25519_dalek::{Signer, SigningKey};
use kortex_abi::{Envelope, Reply, RequestKind};
use kortex_core::{
    ArtifactDescriptor, GuestEngine, Runtime, RuntimeError, VerifiedTool, ai_bridge,
    dispatcher::Clock,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{cell::Cell, collections::BTreeMap, future::Future, rc::Rc};
fn run<T>(f: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    match f.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(v) => v,
        _ => panic!("test future pending"),
    }
}
struct TestClock(Rc<Cell<u64>>);
impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.0.get()
    }
    async fn wait_until(&self, _: u64) {
        std::future::pending::<()>().await
    }
}
struct Engine(Rc<Cell<usize>>);
impl GuestEngine for Engine {
    type Handle = ();
    async fn load(&mut self, _: &VerifiedTool) -> Result<(), RuntimeError> {
        Ok(())
    }
    async fn dispatch(
        &mut self,
        _: &mut (),
        request: &Envelope,
        _: u64,
    ) -> Result<Reply, RuntimeError> {
        let value = if request.kind == RequestKind::ActionInvoke {
            self.0.set(self.0.get() + 1);
            json!({"result":request.payload["input"]["a"].as_f64().unwrap()*request.payload["input"]["b"].as_f64().unwrap()})
        } else {
            json!({})
        };
        Ok(Reply {
            v: 1,
            request_id: request.request_id.clone(),
            ok: true,
            data: Some(value.as_object().unwrap().clone()),
            error: None,
        })
    }
    async fn unload(&mut self, _: ()) -> Result<(), RuntimeError> {
        Ok(())
    }
}
fn tool() -> VerifiedTool {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../tools/calculatrice/manifest.json")).unwrap();
    let contract: Value =
        serde_json::from_str(include_str!("../../../tools/calculatrice/contract.json")).unwrap();
    let bytes = b"signed unit engine fixture, not WASM".to_vec();
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let key = SigningKey::from_bytes(&[7; 32]);
    let text = format!(
        "kortex-artifact-v1\ncalculatrice\n{}\n1.0\n{sha256}\n",
        manifest["version"].as_str().unwrap()
    );
    let proof = ArtifactDescriptor {
        schema_version: 1,
        tool_id: "calculatrice".into(),
        version: manifest["version"].as_str().unwrap().into(),
        abi_version: "1.0".into(),
        sha256,
        signing_key_id: "unit-only".into(),
        signature: key.sign(text.as_bytes()).to_bytes(),
    };
    VerifiedTool::verify(
        &manifest,
        &contract,
        bytes,
        &proof,
        &BTreeMap::from([("unit-only".into(), key.verifying_key().to_bytes())]),
    )
    .unwrap()
}
fn proposal() -> Value {
    json!({"v":1,"summary":"multiply locally","steps":[{"tool_id":"calculatrice","action_id":"multiply","input":{"a":25,"b":48},"requires_consent":true}]})
}
#[test]
fn review_and_deny_never_invoke_and_approval_is_single_use() {
    let calls = Rc::new(Cell::new(0));
    let mut host = Runtime::new(Engine(calls.clone()), TestClock(Rc::new(Cell::new(1))));
    let id = run(host.load(tool())).unwrap();
    run(host.initialize(id)).unwrap();
    let mut review = host.review_ai_plan(&proposal()).unwrap();
    assert_eq!(calls.get(), 0);
    assert!(run(host.execute_ai_step(&mut review, 0, false)).is_err());
    assert_eq!(calls.get(), 0);
    assert!(run(host.execute_ai_step(&mut review, 0, true)).is_err());
    let mut fresh = host.review_ai_plan(&proposal()).unwrap();
    assert_eq!(
        run(host.execute_ai_step(&mut fresh, 0, true)).unwrap()["result"],
        1200.0
    );
    assert_eq!(calls.get(), 1);
    assert!(run(host.execute_ai_step(&mut fresh, 0, true)).is_err());
}
#[test]
fn expiry_unknown_actions_invalid_input_and_unloaded_instance_fail_closed() {
    let now = Rc::new(Cell::new(1));
    let calls = Rc::new(Cell::new(0));
    let mut host = Runtime::new(Engine(calls.clone()), TestClock(now.clone()));
    let id = run(host.load(tool())).unwrap();
    run(host.initialize(id)).unwrap();
    for value in [
        json!({"action_id":"delete-files"}),
        json!({"input":{"a":"25","b":48}}),
        json!({"requires_consent":false}),
    ] {
        let mut p = proposal();
        for (k, v) in value.as_object().unwrap() {
            p["steps"][0][k] = v.clone()
        }
        assert!(host.review_ai_plan(&p).is_err())
    }
    let mut review = host.review_ai_plan(&proposal()).unwrap();
    now.set(300001);
    assert!(run(host.execute_ai_step(&mut review, 0, true)).is_err());
    now.set(1);
    let mut review = host.review_ai_plan(&proposal()).unwrap();
    run(host.shutdown(id)).unwrap();
    assert!(run(host.execute_ai_step(&mut review, 0, true)).is_err());
    assert_eq!(calls.get(), 0);
}
#[test]
fn output_redacts_handles_paths_and_rejects_oversize() {
    let value = ai_bridge::local_result(
        json!({"handle":"private-handle","nested":{"path":"C:\\Users\\private"},"safe":1200}),
    )
    .unwrap();
    assert_eq!(value["safe"], 1200);
    assert!(!value.to_string().contains("private"));
    assert!(ai_bridge::local_result(json!({"text":"a".repeat(32768)})).is_err());
}
