use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use kortex_desktop::{
    config::Config,
    revocation::{Cache, cache_path},
};
pub fn seed(c: &mut Config) {
    let trust = std::fs::read(&c.trust_path).unwrap();
    std::fs::create_dir_all(&c.data_dir).unwrap();
    c.trust_path = c.data_dir.join("fixture-trust.json");
    std::fs::write(&c.trust_path, trust).unwrap();
    let key = SigningKey::from_bytes(&[43; 32]);
    let root = kortex_verify::keys::RootKey {
        id: "lifecycle-test-root".into(),
        public_key_base64: STANDARD.encode(key.verifying_key().as_bytes()),
    };
    std::fs::write(
        c.trust_path.with_file_name("revocation-root.json"),
        serde_json::to_vec(&root).unwrap(),
    )
    .unwrap();
    let now = kortex_runtime_native::permissions::now_ms();
    let mut document = kortex_verify::revocation::Document {
        schema_version: 1,
        version: 1,
        issued_at: now - 1000,
        valid_until: now + 86399000,
        root_key_id: root.id.clone(),
        revoked_keys: vec![],
        revoked_artifacts: vec![],
        signature: String::new(),
    };
    document.signature = STANDARD.encode(key.sign(&document.signing_text().unwrap()).to_bytes());
    let path = cache_path(c, &root);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        serde_json::to_vec(&Cache {
            document,
            seen: now,
        })
        .unwrap(),
    )
    .unwrap();
}
