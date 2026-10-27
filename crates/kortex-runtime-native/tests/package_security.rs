use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::collections::BTreeMap;

#[test]
fn signed_hostile_packages_are_refused_before_engine_creation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let original = std::fs::read(root.join("tests/fixtures/artifact-signed-test.tool")).unwrap();
    let files = kortex_pack::archive::read(&original).unwrap();
    let key = ed25519_dalek::SigningKey::from_bytes(&[23; 32]); // TEST only, never persisted.
    let public = key.verifying_key().to_bytes();
    let mut cases = Vec::new();
    let mut extra = files.clone();
    extra.insert("extra.txt".into(), vec![1]);
    cases.push(("extra-file", extra));
    let mut conflict = files.clone();
    let mut contract: serde_json::Value =
        serde_json::from_slice(&conflict["contract.json"]).unwrap();
    contract["tool_id"] = serde_json::json!("other-tool");
    conflict.insert(
        "contract.json".into(),
        serde_json::to_vec(&contract).unwrap(),
    );
    cases.push(("identity-conflict", conflict));
    for (name, source) in [
        (
            "wasi",
            r#"(module (import "wasi_snapshot_preview1" "fd_write" (func)))"#,
        ),
        (
            "shared-memory",
            r#"(module (memory (export "memory") 1 1024 shared))"#,
        ),
        (
            "unbounded-memory",
            r#"(module (memory (export "memory") 1))"#,
        ),
        (
            "wrong-export",
            r#"(module (memory (export "memory") 1 1024) (func (export "kortex_alloc") (param i64)))"#,
        ),
    ] {
        let mut changed = files.clone();
        changed.insert("module.wasm".into(), wat::parse_str(source).unwrap());
        cases.push((name, changed));
    }
    let mut report = Vec::new();
    for (name, files) in cases {
        // Correct signatures deliberately reach the structural admission boundary.
        let bytes = kortex_pack::archive::write(&files).unwrap();
        let proof = kortex_verify::sign_descriptor(
            &bytes,
            "calculatrice",
            "1.0.0",
            "1.0",
            "hostile-test",
            &[23; 32],
        )
        .unwrap();
        let native = kortex_core::ArtifactDescriptor {
            schema_version: 1,
            tool_id: proof.tool_id.clone(),
            version: proof.version.clone(),
            abi_version: proof.abi_version.clone(),
            sha256: proof.sha256.clone(),
            signing_key_id: proof.signing_key_id.clone(),
            signature: STANDARD
                .decode(&proof.signature)
                .unwrap()
                .try_into()
                .unwrap(),
        };
        assert!(
            kortex_runtime_native::verify_package(
                bytes.clone(),
                &native,
                &BTreeMap::from([("hostile-test".into(), public)])
            )
            .is_err(),
            "{name}"
        );
        report.push(serde_json::json!({"name":name,"archive_base64":STANDARD.encode(bytes),"descriptor":proof}));
    }
    std::fs::create_dir_all(root.join("target")).unwrap();
    std::fs::write(
        root.join("target/malformed-admission.json"),
        serde_json::to_vec(
            &serde_json::json!({"public_key_base64":STANDARD.encode(public),"cases":report}),
        )
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn native_signed_root_admission_and_tampering() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bytes = std::fs::read(root.join("tests/fixtures/artifact-signed-test.tool")).unwrap();
    let proof = kortex_verify::Descriptor::parse(
        &std::fs::read(root.join("tests/fixtures/artifact-signed-test.json")).unwrap(),
    )
    .unwrap();
    let mut bundle = kortex_runtime_native::HostTrust {
        root: serde_json::from_slice(
            &std::fs::read(root.join("shared/fixtures/trust-test-root.json")).unwrap(),
        )
        .unwrap(),
        document: serde_json::from_slice(
            &std::fs::read(root.join("shared/fixtures/trust-test-keys.json")).unwrap(),
        )
        .unwrap(),
        minimum_version: 1,
    };
    assert!(
        kortex_runtime_native::verify_package_with_trust(bytes.clone(), &proof, &bundle, 1000)
            .is_ok()
    );
    bundle.minimum_version = 2;
    assert!(
        kortex_runtime_native::verify_package_with_trust(bytes.clone(), &proof, &bundle, 1000)
            .is_err()
    );
    bundle.minimum_version = 1;
    bundle.document.keys[0].revoked = true;
    assert!(
        kortex_runtime_native::verify_package_with_trust(bytes.clone(), &proof, &bundle, 1000)
            .is_err()
    );
    let mut changed = bytes;
    changed[40] ^= 1;
    assert!(
        kortex_runtime_native::verify_package_with_trust(changed, &proof, &bundle, 1000).is_err()
    );
}
