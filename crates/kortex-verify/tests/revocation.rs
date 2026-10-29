use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use kortex_verify::{Descriptor, keys::RootKey, revocation::Document};
fn signed(version: u64, keys: Vec<String>) -> (Document, RootKey) {
    let key = SigningKey::from_bytes(&[77; 32]);
    let root = RootKey {
        id: "test-offline-root".into(),
        public_key_base64: STANDARD.encode(key.verifying_key().as_bytes()),
    };
    let mut d = Document {
        schema_version: 1,
        version,
        issued_at: 1000,
        valid_until: 86401000,
        root_key_id: root.id.clone(),
        revoked_keys: keys,
        revoked_artifacts: vec![],
        signature: String::new(),
    };
    d.signature = STANDARD.encode(key.sign(&d.signing_text().unwrap()).to_bytes());
    (d, root)
}
#[test]
fn root_signature_tampering_ttl_and_monotonic_tombstones() {
    let (d, root) = signed(2, vec!["compromised".into()]);
    d.verify(&root).unwrap();
    assert!(d.fresh(1000));
    assert!(!d.fresh(86401000));
    let mut forged = d.clone();
    forged.version += 1;
    assert!(forged.verify(&root).is_err());
    let (old, _) = signed(1, vec![]);
    assert!(d.successor(&old));
    assert!(!old.successor(&d));
    let (empty, _) = signed(3, vec![]);
    assert!(!empty.successor(&d));
    let mut long = d.clone();
    long.valid_until += 1;
    assert!(long.signing_text().is_err());
    let mut wrong = root;
    wrong.id = "wrong".into();
    assert!(d.verify(&wrong).is_err());
}
#[test]
fn compromised_signing_key_is_revoked_even_after_snapshot_expiry() {
    let (d, root) = signed(1, vec!["compromised".into()]);
    let p = Descriptor {
        schema_version: 1,
        tool_id: "calculator".into(),
        version: "1.0.0".into(),
        abi_version: "1.0".into(),
        sha256: "a".repeat(64),
        signing_key_id: "compromised".into(),
        signature: String::new(),
    };
    d.verify(&root).unwrap();
    assert!(d.revoked(&p));
    assert!(!d.fresh(86401000));
    assert!(d.revoked(&p));
}
