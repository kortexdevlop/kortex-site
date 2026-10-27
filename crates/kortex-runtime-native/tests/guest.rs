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

#[test]
fn real_calculatrice_signed_archive_and_goldens() {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use kortex_core::ArtifactDescriptor;
    use std::collections::BTreeMap;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = root.join("target/calculatrice-fixture");
    let proof: serde_json::Value = serde_json::from_slice(
        &std::fs::read(dir.join("fixture.json"))
            .expect("Run tests/fixtures/calculatrice.cjs after building the calculator WASM"),
    )
    .unwrap();
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/golden/calculatrice-reference.json"
    ))
    .unwrap();
    assert_eq!(proof["guest_sha256"], reference["guest_sha256"]);
    assert_eq!(proof["descriptor"]["sha256"], reference["archive_sha256"]);
    let d = &proof["descriptor"];
    let descriptor = ArtifactDescriptor {
        schema_version: 1,
        tool_id: d["tool_id"].as_str().unwrap().into(),
        version: d["version"].as_str().unwrap().into(),
        abi_version: "1.0".into(),
        sha256: d["sha256"].as_str().unwrap().into(),
        signing_key_id: d["signing_key_id"].as_str().unwrap().into(),
        signature: STANDARD
            .decode(d["signature"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap(),
    };
    let keys: BTreeMap<String, [u8; 32]> = proof["trust"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(id, key)| {
            (
                id.clone(),
                STANDARD
                    .decode(key.as_str().unwrap())
                    .unwrap()
                    .try_into()
                    .unwrap(),
            )
        })
        .collect();
    let bytes = std::fs::read(dir.join("calculatrice.tool")).unwrap();
    let verified =
        kortex_runtime_native::verify_package(bytes.clone(), &descriptor, &keys).unwrap();
    assert_eq!(verified.manifest().id, "calculatrice");
    let module = std::fs::read(dir.join("module.wasm")).unwrap();
    let compiled = wasmtime::Module::new(&wasmtime::Engine::default(), &module).unwrap();
    assert_eq!(
        compiled.imports().count(),
        0,
        "pure guest must have no sensitive imports"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(&module)),
        proof["guest_sha256"].as_str().unwrap()
    );
    let mut engine = NativeGuestEngine;
    let mut handle = run(engine.load(&verified)).unwrap();
    for line in include_str!("../../../tests/golden/calculatrice.ndjson").lines() {
        let case: serde_json::Value = serde_json::from_str(line).unwrap();
        let request = Envelope::decode(&serde_json::to_vec(&case["request"]).unwrap()).unwrap();
        let reply = run(engine.dispatch(&mut handle, &request, deadline())).unwrap();
        assert_eq!(serde_json::to_value(reply).unwrap(), case["reply"]);
    }
    run(engine.unload(handle)).unwrap();
    let mut bad = bytes;
    bad[40] ^= 1;
    assert!(kortex_runtime_native::verify_package(bad, &descriptor, &keys).is_err());
}
