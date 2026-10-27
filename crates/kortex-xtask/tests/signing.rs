use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use kortex_verify::keys::{RootKey, SignedKeys, TrustKey};
use std::process::Command;
#[test]
fn cli_signs_from_injected_test_secret_and_verifies_public_proof() {
    let temp = tempfile::tempdir().unwrap();
    let archive = temp.path().join("sample.tool");
    std::fs::write(
        &archive,
        include_bytes!("../../../tests/fixtures/artifact-signed-test.tool"),
    )
    .unwrap();
    let secret = [93; 32];
    let key = SigningKey::from_bytes(&secret);
    let root = SigningKey::from_bytes(&[94; 32]);
    let anchor = RootKey {
        id: "ci-test-root".into(),
        public_key_base64: STANDARD.encode(root.verifying_key().to_bytes()),
    };
    let mut store = SignedKeys {
        schema_version: 1,
        version: 2,
        issued_at: 0,
        expires: 4102444800000,
        root_key_id: anchor.id.clone(),
        keys: vec![TrustKey {
            id: "ci-test-only".into(),
            public_key_base64: STANDARD.encode(key.verifying_key().to_bytes()),
            not_before: 0,
            expires: 4102444800000,
            revoked: false,
        }],
        signature: String::new(),
    };
    store.signature = STANDARD.encode(root.sign(&store.signing_text().unwrap()).to_bytes());
    let root_path = temp.path().join("root.json");
    let keys_path = temp.path().join("keys.json");
    std::fs::write(&root_path, serde_json::to_vec(&anchor).unwrap()).unwrap();
    std::fs::write(&keys_path, serde_json::to_vec(&store).unwrap()).unwrap();
    let signed = Command::new(env!("CARGO_BIN_EXE_kortex-xtask"))
        .env("CI", "true")
        .env("KORTEX_SIGNING_KEY_ID", "ci-test-only")
        .env("KORTEX_SIGNING_KEY_SEED_BASE64", STANDARD.encode(secret))
        .args(["sign", "--artifact"])
        .arg(&archive)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        signed.status.success(),
        "{}",
        String::from_utf8_lossy(&signed.stderr)
    );
    assert!(!String::from_utf8_lossy(&signed.stdout).contains(&STANDARD.encode(secret)));
    let proof = temp.path().join("sample.tool.json");
    let verify = |minimum: &str| {
        Command::new(env!("CARGO_BIN_EXE_kortex-xtask"))
            .args(["verify", "--artifact"])
            .arg(&proof)
            .arg("--root")
            .arg(&root_path)
            .arg("--trust")
            .arg(&keys_path)
            .args(["--min-trust-version", minimum, "--json"])
            .output()
            .unwrap()
    };
    let valid = verify("2");
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&valid.stdout).unwrap()["signature_verified"],
        true
    );
    assert!(!verify("3").status.success());
    let mut bytes = std::fs::read(&archive).unwrap();
    bytes[40] ^= 1;
    std::fs::write(&archive, bytes).unwrap();
    assert!(!verify("2").status.success());
}
