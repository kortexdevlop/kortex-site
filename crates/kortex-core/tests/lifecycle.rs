use ed25519_dalek::{Signer, SigningKey};
use kortex_abi::{Envelope, Reply, RequestKind};
use kortex_core::{dispatcher::Clock, *};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    future::Future,
    rc::Rc,
};
fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    for _ in 0..1000 {
        if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
    }
    panic!("unit future did not complete")
}
fn material() -> (
    Value,
    Value,
    Vec<u8>,
    ArtifactDescriptor,
    BTreeMap<String, [u8; 32]>,
) {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../shared/fixtures/manifest.valid.json")).unwrap();
    let mut contract: Value =
        serde_json::from_str(include_str!("../../../shared/fixtures/contract.valid.json")).unwrap();
    contract["actions"][0]["timeout_ms"] = json!(30000);
    let bytes = b"explicitly signed unit fixture; engine stub does not extract ZIP".to_vec();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let signing = SigningKey::from_bytes(&[7; 32]);
    let text = format!("kortex-artifact-v1\ndemo-calculator\n1.0.0\n1.0\n{digest}\n");
    let proof = ArtifactDescriptor {
        schema_version: 1,
        tool_id: "demo-calculator".into(),
        version: "1.0.0".into(),
        abi_version: "1.0".into(),
        sha256: digest,
        signing_key_id: "unit-only".into(),
        signature: signing.sign(text.as_bytes()).to_bytes(),
    };
    let keys = BTreeMap::from([("unit-only".into(), signing.verifying_key().to_bytes())]);
    (manifest, contract, bytes, proof, keys)
}
fn tool() -> VerifiedTool {
    let (m, c, b, p, k) = material();
    VerifiedTool::verify(&m, &c, b, &p, &k).unwrap()
}
#[derive(Clone, Default)]
struct FakeClock {
    now: Rc<Cell<u64>>,
    deadlines: Rc<RefCell<Vec<u64>>>,
}
impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.now.get()
    }
    async fn wait_until(&self, deadline: u64) {
        self.deadlines.borrow_mut().push(deadline);
        self.now.set(deadline);
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Hang,
    Crash,
    BadCorrelation,
    BadOutput,
    ReportedError,
}
#[derive(Clone)]
struct Controls {
    mode: Rc<Cell<Mode>>,
    fail_unload: Rc<Cell<bool>>,
    loads: Rc<Cell<usize>>,
    unloads: Rc<Cell<usize>>,
    cancelled: Rc<Cell<usize>>,
    requests: Rc<RefCell<Vec<RequestKind>>>,
}
impl Default for Controls {
    fn default() -> Self {
        Self {
            mode: Rc::new(Cell::new(Mode::Normal)),
            fail_unload: Default::default(),
            loads: Default::default(),
            unloads: Default::default(),
            cancelled: Default::default(),
            requests: Default::default(),
        }
    }
}
struct Engine {
    control: Controls,
}
struct Cancelled(Rc<Cell<usize>>);
impl Drop for Cancelled {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1)
    }
}
impl GuestEngine for Engine {
    type Handle = usize;
    async fn load(&mut self, _: &VerifiedTool) -> Result<usize, RuntimeError> {
        let n = self.control.loads.get() + 1;
        self.control.loads.set(n);
        Ok(n)
    }
    async fn dispatch(
        &mut self,
        _: &mut usize,
        request: &Envelope,
        _deadline: u64,
    ) -> Result<Reply, RuntimeError> {
        self.control
            .requests
            .borrow_mut()
            .push(request.kind.clone());
        if self.control.mode.get() == Mode::Hang {
            let _guard = Cancelled(self.control.cancelled.clone());
            std::future::pending::<()>().await;
        }
        if self.control.mode.get() == Mode::Crash {
            return Err(RuntimeError::GuestCrash);
        }
        let data = if request.kind == RequestKind::ActionInvoke {
            if self.control.mode.get() == Mode::BadOutput {
                json!({"result":"wrong"})
            } else {
                json!({"result":request.payload["input"]["a"].as_i64().unwrap_or(0)+request.payload["input"]["b"].as_i64().unwrap_or(0)})
            }
        } else {
            json!({})
        };
        let mut reply = Reply {
            v: 1,
            request_id: request.request_id.clone(),
            ok: true,
            data: Some(data.as_object().unwrap().clone()),
            error: None,
        };
        if self.control.mode.get() == Mode::BadCorrelation {
            reply.request_id = "00000000-0000-4000-8000-000000000000".into()
        }
        if self.control.mode.get() == Mode::ReportedError {
            reply.ok = false;
            reply.data = None;
            reply.error = Some(kortex_abi::HostError {
                code: "ACTION_FAILED".into(),
                message: "failed".into(),
                retryable: false,
            })
        }
        Ok(reply)
    }
    async fn unload(&mut self, _: usize) -> Result<(), RuntimeError> {
        self.control.unloads.set(self.control.unloads.get() + 1);
        if self.control.fail_unload.get() {
            return Err(RuntimeError::Engine("unload failure".into()));
        }
        Ok(())
    }
}
fn runtime() -> (Runtime<Engine, FakeClock>, Controls, FakeClock) {
    let control = Controls::default();
    let clock = FakeClock::default();
    (
        Runtime::new(
            Engine {
                control: control.clone(),
            },
            clock.clone(),
        ),
        control,
        clock,
    )
}
fn running(rt: &mut Runtime<Engine, FakeClock>) -> InstanceId {
    let id = block_on(rt.load(tool())).unwrap();
    block_on(rt.initialize(id)).unwrap();
    id
}
#[test]
fn lifecycle_accepts_only_declared_transitions() {
    let mut state = State::Created;
    assert_eq!(
        state.transition(State::Running),
        Err(RuntimeError::InvalidState)
    );
    for next in [
        State::Loaded,
        State::Initialized,
        State::Running,
        State::Stopped,
    ] {
        state.transition(next).unwrap()
    }
    assert_eq!(
        state.transition(State::Loaded),
        Err(RuntimeError::InvalidState)
    );
    let mut failure = State::Created;
    failure.transition(State::Error).unwrap();
    assert!(failure.transition(State::Running).is_err());
}
#[test]
fn verification_rejects_unsigned_tampered_and_untrusted_material() {
    let (m, c, b, mut p, k) = material();
    assert!(VerifiedTool::verify(&m, &c, b.clone(), &p, &k).is_ok());
    p.signature = [0; 64];
    assert!(matches!(
        VerifiedTool::verify(&m, &c, b.clone(), &p, &k),
        Err(RuntimeError::SignatureInvalid)
    ));
    let (m, c, b, p, k) = material();
    assert!(matches!(
        VerifiedTool::verify(&m, &c, b"tampered".to_vec(), &p, &k),
        Err(RuntimeError::IntegrityError)
    ));
    assert!(matches!(
        VerifiedTool::verify(&m, &c, b, &p, &BTreeMap::new()),
        Err(RuntimeError::UntrustedKey)
    ));
}
#[test]
fn handshake_actions_events_and_shutdown() {
    let (mut rt, c, _) = runtime();
    let id = block_on(rt.load(tool())).unwrap();
    assert_eq!(rt.instance(id).unwrap().state(), State::Loaded);
    assert_eq!(
        block_on(rt.invoke_action(id, "add", json!({"a":1,"b":2}))),
        Err(RuntimeError::InvalidState)
    );
    block_on(rt.initialize(id)).unwrap();
    assert_eq!(rt.instance(id).unwrap().state(), State::Running);
    assert_eq!(block_on(rt.initialize(id)), Err(RuntimeError::InvalidState));
    let reply = block_on(rt.invoke_action(id, "add", json!({"a":1,"b":2}))).unwrap();
    assert_eq!(reply.data.unwrap()["result"], 3);
    block_on(rt.dispatch_event(
        id,
        json!({"node_key":"button","event_type":"click","value":null}),
    ))
    .unwrap();
    block_on(rt.shutdown(id)).unwrap();
    assert_eq!(rt.instance_count(), 0);
    assert_eq!(c.unloads.get(), 1);
    assert_eq!(
        block_on(rt.shutdown(id)),
        Err(RuntimeError::UnknownInstance)
    );
    assert_eq!(
        &c.requests.borrow()[..2],
        &[RequestKind::Init, RequestKind::Render]
    );
}
#[test]
fn queue_128_fifo_and_backpressure() {
    let (mut rt, _, _) = runtime();
    let id = running(&mut rt);
    let mut ids = Vec::new();
    for n in 0..128 {
        ids.push(rt.enqueue_action(id, "add", json!({"a":n,"b":1})).unwrap());
    }
    assert_eq!(rt.instance(id).unwrap().queued(), 128);
    assert_eq!(
        rt.enqueue_action(id, "add", json!({"a":1,"b":2})),
        Err(RuntimeError::QueueFull)
    );
    assert_eq!(
        block_on(rt.invoke_action(id, "add", json!({"a":1,"b":2}))),
        Err(RuntimeError::QueueBusy)
    );
    for (n, request_id) in ids.iter().enumerate() {
        let reply = block_on(rt.process_next(id)).unwrap().unwrap();
        assert_eq!(&reply.request_id, request_id);
        assert_eq!(reply.data.unwrap()["result"], n + 1);
    }
    assert!(block_on(rt.process_next(id)).unwrap().is_none());
    block_on(rt.shutdown(id)).unwrap();
}
#[test]
fn timeout_30_seconds_cancels_future_and_cleans_instance() {
    let (mut rt, c, clock) = runtime();
    let id = running(&mut rt);
    c.mode.set(Mode::Hang);
    assert_eq!(
        block_on(rt.invoke_action(id, "add", json!({"a":1,"b":2}))),
        Err(RuntimeError::Timeout)
    );
    assert_eq!(clock.now.get(), 30000);
    assert_eq!(*clock.deadlines.borrow(), vec![30000]);
    assert_eq!(c.cancelled.get(), 1);
    assert_eq!(c.unloads.get(), 1);
    assert_eq!(rt.instance_count(), 0);
}
#[test]
fn crash_correlation_and_bad_output_are_fatal_and_clean() {
    for (mode, error) in [
        (Mode::Crash, RuntimeError::GuestCrash),
        (Mode::BadCorrelation, RuntimeError::CorrelationError),
        (Mode::BadOutput, RuntimeError::InvalidOutput),
    ] {
        let (mut rt, c, _) = runtime();
        let id = running(&mut rt);
        c.mode.set(mode);
        assert_eq!(
            block_on(rt.invoke_action(id, "add", json!({"a":1,"b":2}))),
            Err(error)
        );
        assert_eq!(rt.instance_count(), 0);
        assert_eq!(c.unloads.get(), 1);
    }
}
#[test]
fn ordinary_guest_error_is_reported_without_losing_instance() {
    let (mut rt, c, _) = runtime();
    let id = running(&mut rt);
    c.mode.set(Mode::ReportedError);
    let reply = block_on(rt.invoke_action(id, "add", json!({"a":1,"b":2}))).unwrap();
    assert!(!reply.ok);
    assert_eq!(reply.error.unwrap().code, "ACTION_FAILED");
    assert_eq!(rt.instance(id).unwrap().state(), State::Running);
    c.mode.set(Mode::Normal);
    block_on(rt.shutdown(id)).unwrap();
    assert_eq!(c.unloads.get(), 1);
}
#[test]
fn instances_remain_isolated_and_invalid_input_does_not_dispatch() {
    let (mut rt, c, _) = runtime();
    let first = running(&mut rt);
    let second = running(&mut rt);
    let before = c.requests.borrow().len();
    assert_eq!(
        rt.enqueue_action(first, "missing", json!({})),
        Err(RuntimeError::InvalidInput)
    );
    assert_eq!(
        rt.enqueue_action(first, "add", json!({"a":"bad","b":2})),
        Err(RuntimeError::InvalidInput)
    );
    assert_eq!(
        block_on(rt.dispatch_event(first, json!({"style":"url(x)"}))),
        Err(RuntimeError::InvalidInput)
    );
    assert_eq!(c.requests.borrow().len(), before);
    block_on(rt.shutdown(first)).unwrap();
    assert_eq!(rt.instance_count(), 1);
    assert_eq!(rt.instance(second).unwrap().state(), State::Running);
    block_on(rt.shutdown(second)).unwrap();
    assert_eq!(rt.instance_count(), 0);
    assert_eq!(c.unloads.get(), 2);
}

#[test]
fn shutdown_error_still_removes_and_unloads() {
    let (mut rt, c, _) = runtime();
    let id = running(&mut rt);
    c.mode.set(Mode::Crash);
    assert_eq!(block_on(rt.shutdown(id)), Err(RuntimeError::GuestCrash));
    assert_eq!(rt.instance_count(), 0);
    assert_eq!(c.unloads.get(), 1);
    let (mut rt, c, _) = runtime();
    let id = running(&mut rt);
    c.mode.set(Mode::ReportedError);
    assert!(matches!(
        block_on(rt.shutdown(id)),
        Err(RuntimeError::GuestError(_))
    ));
    assert_eq!(rt.instance_count(), 0);
    assert_eq!(c.unloads.get(), 1);
}

#[test]
fn cleanup_failure_is_visible_and_registry_is_empty() {
    let (mut rt, c, _) = runtime();
    let id = running(&mut rt);
    c.mode.set(Mode::Crash);
    c.fail_unload.set(true);
    assert!(matches!(
        block_on(rt.invoke_action(id, "add", json!({"a":1,"b":2}))),
        Err(RuntimeError::CleanupFailure { .. })
    ));
    assert_eq!(rt.instance_count(), 0);
    assert_eq!(c.unloads.get(), 1);
}

#[test]
fn runtime_derives_identity_and_declarations_for_every_capability() {
    use kortex_capabilities::{
        AuthorizedCall, Capability, CapabilityCall, CapabilityError, CapabilityHost,
        CapabilityResponse, Gateway, Resource, policy::Duration,
    };
    struct Sensitive(Rc<Cell<usize>>);
    impl CapabilityHost for Sensitive {
        async fn execute(
            &mut self,
            _: &AuthorizedCall<'_>,
        ) -> Result<CapabilityResponse, CapabilityError> {
            self.0.set(self.0.get() + 1);
            Ok(CapabilityResponse::Bytes(vec![]))
        }
    }
    let (mut rt, _, _) = runtime();
    let (m, c, b, p, k) = material();
    let mut manifest = m;
    manifest["permissions"] = json!(["storage.read"]);
    let tool = VerifiedTool::verify(&manifest, &c, b, &p, &k).unwrap();
    let id = block_on(rt.load(tool)).unwrap();
    block_on(rt.initialize(id)).unwrap();
    let calls = Rc::new(Cell::new(0));
    let mut gateway = Gateway::new(Sensitive(calls.clone()));
    let context = rt.prepare_capabilities(id, &mut gateway).unwrap();
    let resource = Resource::Storage {
        key: "preferences".into(),
    };
    let make = |capability| CapabilityCall {
        request_id: "00000000-0000-4000-8000-000000000001".into(),
        capability,
        resource: resource.clone(),
        bytes: vec![],
        max_output_bytes: 100,
    };
    assert!(matches!(
        block_on(rt.execute_capability(id, &mut gateway, make(Capability::StorageRead), None)),
        Err(CapabilityError::Denied)
    ));
    assert_eq!(calls.get(), 0);
    gateway
        .policy
        .grant(
            &context,
            Capability::StorageRead,
            &resource,
            Duration::Session,
            1000,
        )
        .unwrap();
    assert!(
        block_on(rt.execute_capability(id, &mut gateway, make(Capability::StorageRead), None))
            .is_ok()
    );
    assert_eq!(calls.get(), 1);
    assert!(
        block_on(rt.execute_capability(id, &mut gateway, make(Capability::StorageWrite), None))
            .is_err()
    );
    assert_eq!(calls.get(), 1);
    block_on(rt.dispatch_capability_result(
        id,
        "00000000-0000-4000-8000-000000000001".into(),
        Ok(Default::default()),
    ))
    .unwrap();
    let large = serde_json::json!({"bytes":"x".repeat(kortex_abi::MAX_MESSAGE_BYTES)})
        .as_object()
        .unwrap()
        .clone();
    assert_eq!(
        block_on(rt.dispatch_capability_result(
            id,
            "00000000-0000-4000-8000-000000000001".into(),
            Ok(large)
        )),
        Err(RuntimeError::InvalidInput)
    );
    assert_eq!(rt.instance(id).unwrap().queued(), 0);
    block_on(rt.shutdown_with_capabilities(id, &mut gateway)).unwrap();
    assert!(
        !gateway
            .policy
            .is_declared(&context, Capability::StorageRead)
    );
    assert!(
        block_on(rt.execute_capability(id, &mut gateway, make(Capability::StorageRead), None))
            .is_err()
    );
    assert_eq!(calls.get(), 1);
}
