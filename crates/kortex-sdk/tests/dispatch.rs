use kortex_sdk::{
    ffi::{BufferArena, BufferError},
    *,
};
use serde_json::{Map, json};
use std::sync::{Arc, Mutex};
fn id(n: u32) -> String {
    format!("00000000-0000-4000-8000-{n:012}")
}
fn request(n: u32, kind: RequestKind, payload: serde_json::Value) -> Request {
    Request {
        v: 1,
        request_id: id(n),
        kind,
        payload: payload.as_object().unwrap().clone(),
    }
}
fn invoke<T: Tool>(dispatcher: &mut Dispatcher<T>, request: Request) -> Reply {
    Reply::decode(&dispatcher.dispatch(&request.encode().unwrap())).unwrap()
}
struct Calculator;
impl Tool for Calculator {
    fn handle(&mut self, request: Request) -> Reply {
        let a = request.payload.get("a").and_then(|v| v.as_i64());
        let b = request.payload.get("b").and_then(|v| v.as_i64());
        match (a, b) {
            (Some(a), Some(b)) => success(
                request.request_id,
                json!({"result":a*b}).as_object().unwrap().clone(),
            ),
            _ => failure(request.request_id, "INVALID_INPUT"),
        }
    }
}
#[test]
fn multiple_actions_and_validation() {
    let mut dispatcher = Dispatcher::new(Calculator);
    for n in 1..=1000 {
        let r = invoke(
            &mut dispatcher,
            request(n, RequestKind::ActionInvoke, json!({"a":25,"b":48})),
        );
        assert_eq!(r.data.unwrap()["result"], 1200);
        assert_eq!(r.request_id, id(n));
    }
    assert!(
        !invoke(
            &mut dispatcher,
            request(1001, RequestKind::ActionInvoke, json!({"a":"bad"}))
        )
        .ok
    );
    assert!(!Reply::decode(&dispatcher.dispatch(b"{")).unwrap().ok);
    assert!(
        !Reply::decode(&dispatcher.dispatch(&vec![b' '; 1048577]))
            .unwrap()
            .ok
    );
}
struct BadTool;
impl Tool for BadTool {
    fn handle(&mut self, _: Request) -> Reply {
        success("bad-id".into(), Map::new())
    }
}
#[test]
fn invalid_tool_reply_is_structured() {
    let mut dispatcher = Dispatcher::new(BadTool);
    let r = invoke(&mut dispatcher, request(1, RequestKind::Render, json!({})));
    assert_eq!(r.error.unwrap().code, "CORRELATION_ERROR");
}
struct Panicking;
impl Tool for Panicking {
    fn handle(&mut self, _: Request) -> Reply {
        panic!("unit-test-only panic")
    }
}
#[test]
fn native_tool_panic_is_caught_before_ffi() {
    let mut dispatcher = Dispatcher::new(Panicking);
    let r = invoke(&mut dispatcher, request(1, RequestKind::Render, json!({})));
    assert_eq!(r.error.unwrap().code, "TOOL_PANIC");
}
#[test]
fn buffer_ownership_and_pointer_length_fuzz() {
    let mut arena = BufferArena::default();
    let pointer = arena.allocate(8).unwrap();
    arena.write(pointer, b"12345678").unwrap();
    assert_eq!(arena.read(pointer, 8).unwrap(), b"12345678");
    assert_eq!(
        arena.deallocate(pointer, 7),
        Err(BufferError::LengthMismatch)
    );
    assert_eq!(arena.live_bytes(), 8);
    let mut seed = 0x76543210u32;
    for _ in 0..20000 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let p = seed;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let n = seed;
        assert!(arena.read(p, n).is_err());
        assert!(arena.deallocate(p, n).is_err());
    }
    assert_eq!(arena.read(pointer, 0), Err(BufferError::LengthMismatch));
    assert_eq!(arena.allocate(1048577), Err(BufferError::InvalidLength));
    assert_eq!(arena.allocate(0), Err(BufferError::InvalidLength));
    arena.deallocate(pointer, 8).unwrap();
    assert_eq!(
        arena.deallocate(pointer, 8),
        Err(BufferError::UnknownAllocation)
    );
    assert_eq!(arena.live_bytes(), 0);
}
fn accepted(_: &[u8]) -> i32 {
    0
}
fn denied(_: &[u8]) -> i32 {
    -1
}
fn badstatus(_: &[u8]) -> i32 {
    1
}
fn capability(n: u32) -> CapabilityRequest {
    CapabilityRequest {
        request_id: id(n),
        capability: "storage.read".into(),
        payload: Map::new(),
    }
}
#[test]
fn pending_results_out_of_order_and_no_replay() {
    let mut host = Host::new(accepted);
    let output = Arc::new(Mutex::new(Vec::new()));
    for n in [1, 2] {
        let pending = host.enqueue(capability(n)).unwrap();
        let target = output.clone();
        host.on_result(&pending, move |result| {
            target
                .lock()
                .unwrap()
                .push(result.unwrap()["number"].as_u64().unwrap())
        })
        .unwrap();
    }
    for n in [2, 1] {
        let r = request(
            100 + n,
            RequestKind::CapabilityResult,
            json!({"capability_request_id":id(n),"ok":true,"data":{"number":n}}),
        );
        host.resolve(&r).unwrap();
        assert!(host.resolve(&r).is_err());
    }
    assert_eq!(*output.lock().unwrap(), vec![2, 1]);
    assert_eq!(host.pending_count(), 0);
    let unknown = request(
        104,
        RequestKind::CapabilityResult,
        json!({"capability_request_id":id(99),"ok":true}),
    );
    assert!(host.resolve(&unknown).is_err());
}
#[test]
fn queue_limits_and_host_rejections() {
    let mut host = Host::new(accepted);
    host.enqueue(capability(1)).unwrap();
    assert_eq!(
        host.enqueue(capability(1)).unwrap_err().code,
        "DUPLICATE_REQUEST"
    );
    for n in 2..=128 {
        host.enqueue(capability(n)).unwrap();
    }
    assert_eq!(
        host.enqueue(capability(129)).unwrap_err().code,
        "QUEUE_FULL"
    );
    for (transport, code) in [
        (denied as fn(&[u8]) -> i32, "CAPABILITY_DENIED"),
        (badstatus, "INVALID_HOST_STATUS"),
    ] {
        let mut host = Host::new(transport);
        assert_eq!(host.enqueue(capability(1)).unwrap_err().code, code);
        assert_eq!(host.pending_count(), 0);
    }
}
struct WithHost {
    host: Host,
}
impl Tool for WithHost {
    fn handle(&mut self, r: Request) -> Reply {
        success(r.request_id, Map::new())
    }
    fn host(&mut self) -> Option<&mut Host> {
        Some(&mut self.host)
    }
}
#[test]
fn dispatcher_correlates_async_results() {
    let mut host = Host::new(accepted);
    host.enqueue(capability(3)).unwrap();
    let mut dispatcher = Dispatcher::new(WithHost { host });
    let r = invoke(
        &mut dispatcher,
        request(
            100,
            RequestKind::CapabilityResult,
            json!({"capability_request_id":id(3),"ok":false,"error":{"code":"DENIED","message":"Denied","retryable":false}}),
        ),
    );
    assert!(r.ok);
    assert_eq!(r.request_id, id(100));
    assert_eq!(dispatcher.tool.host.pending_count(), 0);
}
