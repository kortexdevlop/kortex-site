use ed25519_dalek::{Signer, SigningKey};
use kortex_abi::{Envelope, RequestKind};
use kortex_core::{ArtifactDescriptor, GuestEngine, RuntimeError, VerifiedTool};
use kortex_runtime_native::{NativeGuestEngine, verify_package};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    future::Future,
    io::{Cursor, Write},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn run<T>(f: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn deadline() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 30000
}
fn tool(module: &[u8]) -> VerifiedTool {
    let manifest: Value =
        serde_json::from_str(include_str!("../../../shared/fixtures/manifest.valid.json")).unwrap();
    let mut contract: Value =
        serde_json::from_str(include_str!("../../../shared/fixtures/contract.valid.json")).unwrap();
    contract["actions"][0]["id"] = json!("multiply");
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in [
        ("manifest.json", serde_json::to_vec(&manifest).unwrap()),
        ("contract.json", serde_json::to_vec(&contract).unwrap()),
        ("module.wasm", module.to_vec()),
    ] {
        zip.start_file(name, options).unwrap();
        zip.write_all(&data).unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let key = SigningKey::from_bytes(&[42; 32]);
    let signature = key
        .sign(format!("kortex-artifact-v1\ndemo-calculator\n1.0.0\n1.0\n{sha256}\n").as_bytes())
        .to_bytes();
    let descriptor = ArtifactDescriptor {
        schema_version: 1,
        tool_id: "demo-calculator".into(),
        version: "1.0.0".into(),
        abi_version: "1.0".into(),
        sha256,
        signing_key_id: "test-only".into(),
        signature,
    };
    verify_package(
        bytes,
        &descriptor,
        &BTreeMap::from([("test-only".into(), key.verifying_key().to_bytes())]),
    )
    .unwrap()
}
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
