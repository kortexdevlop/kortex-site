mod common;
use common::{deadline, run, tool};
use kortex_abi::{Envelope, RequestKind};
use kortex_core::{GuestEngine, RuntimeError};
use kortex_runtime_native::NativeGuestEngine;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
#[test]
fn same_guest_checksum_and_native_multiply() {
    let bytes=std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"),"/../../tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65"
    );
    let mut engine = NativeGuestEngine;
    let verified = tool(&bytes);
    let mut handle = run(engine.load(&verified)).unwrap();
    for i in 0..100 {
        let request = Envelope {
            v: 1,
            request_id: format!("00000000-0000-4000-8000-{i:012x}"),
            kind: RequestKind::ActionInvoke,
            payload: json!({"action_id":"multiply","input":{"a":25,"b":48}})
                .as_object()
                .unwrap()
                .clone(),
        };
        let reply = run(engine.dispatch(&mut handle, &request, deadline())).unwrap();
        assert_eq!(reply.data.unwrap()["result"], 1200);
    }
    run(engine.unload(handle)).unwrap();
}
#[test]
fn loop_and_invalid_buffer_are_fatal() {
    for (body, expected) in [
        ("(loop br 0) i64.const 0", RuntimeError::Timeout),
        ("i64.const 281449206906980", RuntimeError::InvalidReply),
    ] {
        let module=wat::parse_str(format!("(module (memory (export \"memory\") 1 1024) (func (export \"kortex_alloc\") (param i32) (result i32) i32.const 64) (func (export \"kortex_dealloc\") (param i32 i32)) (func (export \"kortex_dispatch\") (param i32 i32) (result i64) {body}))")).unwrap();
        let mut engine = NativeGuestEngine;
        let mut handle = run(NativeGuestEngine::instantiate(tool(&module))).unwrap();
        let request = Envelope {
            v: 1,
            request_id: "00000000-0000-4000-8000-000000000000".into(),
            kind: RequestKind::Init,
            payload: Default::default(),
        };
        let start = Instant::now();
        assert_eq!(
            run(engine.dispatch(&mut handle, &request, deadline() - 29900)),
            Err(expected)
        );
        assert!(start.elapsed() < Duration::from_secs(5));
    }
}

#[test]
fn cancelled_dispatch_interrupts_actor_and_can_be_unloaded() {
    let module=wat::parse_str("(module (memory (export \"memory\") 1 1024) (func (export \"kortex_alloc\") (param i32) (result i32) i32.const 64) (func (export \"kortex_dealloc\") (param i32 i32)) (func (export \"kortex_dispatch\") (param i32 i32) (result i64) (loop br 0) i64.const 0))").unwrap();
    let mut engine = NativeGuestEngine;
    let mut handle = run(engine.load(&tool(&module))).unwrap();
    let request = Envelope {
        v: 1,
        request_id: "00000000-0000-4000-8000-000000000090".into(),
        kind: RequestKind::Init,
        payload: Default::default(),
    };
    let started = Instant::now();
    let mut pending = Box::pin(engine.dispatch(&mut handle, &request, deadline()));
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    assert!(std::future::Future::poll(pending.as_mut(), &mut cx).is_pending());
    drop(pending);
    assert_eq!(
        run(engine.dispatch(&mut handle, &request, deadline())),
        Err(RuntimeError::GuestCrash)
    );
    run(engine.unload(handle)).unwrap();
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn signed_text_module_is_not_a_portable_wasm_binary() {
    let mut engine = NativeGuestEngine;
    assert!(matches!(
        run(engine.load(&tool(b"(module)"))),
        Err(RuntimeError::UnsupportedAbi)
    ));
}
