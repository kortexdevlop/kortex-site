#![allow(dead_code)]
use ed25519_dalek::{Signer, SigningKey};

use kortex_core::{ArtifactDescriptor, VerifiedTool};
use kortex_runtime_native::verify_package;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    future::Future,
    io::{Cursor, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
pub fn run<T>(f: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
pub fn deadline() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 30000
}
pub fn tool(module: &[u8]) -> VerifiedTool {
    tool_with(module, &[])
}
pub fn tool_with(module: &[u8], permissions: &[&str]) -> VerifiedTool {
    let mut manifest: Value = serde_json::from_str(include_str!(
        "../../../../shared/fixtures/manifest.valid.json"
    ))
    .unwrap();
    manifest["permissions"] = json!(permissions);
    let mut contract: Value = serde_json::from_str(include_str!(
        "../../../../shared/fixtures/contract.valid.json"
    ))
    .unwrap();
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
