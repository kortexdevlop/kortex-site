use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use kortex_verify::{keys::*, *};
fn document(keys: Vec<TrustKey>, version: u64) -> (SignedKeys, RootKey) {
    let root = SigningKey::from_bytes(&[9; 32]);
    let anchor = RootKey {
        id: "root-test".into(),
        public_key_base64: STANDARD.encode(root.verifying_key().to_bytes()),
    };
    let mut d = SignedKeys {
        schema_version: 1,
        version,
        issued_at: 0,
        expires: 10000,
        root_key_id: anchor.id.clone(),
        keys,
        signature: String::new(),
    };
    d.signature = STANDARD.encode(root.sign(&d.signing_text().unwrap()).to_bytes());
    (d, anchor)
}
fn key() -> TrustKey {
    TrustKey {
        id: "leaf-test".into(),
        public_key_base64: STANDARD
            .encode(SigningKey::from_bytes(&[7; 32]).verifying_key().to_bytes()),
        not_before: 0,
        expires: 5000,
        revoked: false,
    }
}
#[test]
fn exact_b2_and_hash_before_signature() {
    let bytes = b"archive bytes";
    let proof =
        sign_descriptor(bytes, "sample-tool", "1.2.3", "1.0", "leaf-test", &[7; 32]).unwrap();
    assert_eq!(
        String::from_utf8(proof.signing_text().unwrap()).unwrap(),
        format!(
            "kortex-artifact-v1\nsample-tool\n1.2.3\n1.0\n{}\n",
            sha256(bytes)
        )
    );
    let (d, r) = document(vec![key()], 1);
    let trust = TrustStore::from_signed(&d, &r, 1, 100).unwrap();
    verify_descriptor(bytes, &proof, &trust, 100).unwrap();
    let mut altered = proof.clone();
    altered.signing_key_id = "unknown".into();
    assert_eq!(
        verify_descriptor(b"altered bytes", &altered, &trust, 100),
        Err(Error::HashMismatch)
    );
    for (field, value) in [
        ("tool_id", "different-tool"),
        ("version", "2.0.0"),
        ("abi_version", "2.0"),
        (
            "signature",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        ),
    ] {
        let mut p = serde_json::to_value(&proof).unwrap();
        p[field] = serde_json::json!(value);
        let p: Descriptor = serde_json::from_value(p).unwrap();
        assert!(
            verify_descriptor(bytes, &p, &trust, 100).is_err(),
            "{field}"
        );
    }
}
#[test]
fn unknown_removed_revoked_expired_keys_and_signed_store_rollback() {
    let bytes = b"archive bytes";
    let p = sign_descriptor(bytes, "sample-tool", "1.0.0", "1.0", "leaf-test", &[7; 32]).unwrap();
    for keys in [
        vec![],
        vec![TrustKey {
            revoked: true,
            ..key()
        }],
        vec![TrustKey {
            expires: 50,
            ..key()
        }],
    ] {
        let (d, r) = document(keys, 2);
        let trust = TrustStore::from_signed(&d, &r, 2, 100).unwrap();
        assert_eq!(
            verify_descriptor(bytes, &p, &trust, 100),
            Err(Error::UntrustedKey)
        );
    }
    let (d, r) = document(vec![key()], 2);
    assert!(matches!(
        TrustStore::from_signed(&d, &r, 3, 100),
        Err(Error::Rollback)
    ));
    assert!(TrustStore::from_signed(&d, &r, 1, 10000).is_err());
    let mut forged = d.clone();
    forged.version = 3;
    assert!(TrustStore::from_signed(&forged, &r, 1, 100).is_err());
    let mut forged = d.clone();
    forged.keys[0].revoked = true;
    assert!(TrustStore::from_signed(&forged, &r, 1, 100).is_err());
    let (_, wrong) = document(vec![], 1);
    let wrong = RootKey {
        public_key_base64: STANDARD.encode([0; 32]),
        ..wrong
    };
    assert!(TrustStore::from_signed(&d, &wrong, 1, 100).is_err());
}
#[test]
fn descriptor_schema_rejects_unknown_fields_injection_and_encoding() {
    let p = sign_descriptor(b"a", "sample-tool", "1.0.0", "1.0", "leaf-test", &[7; 32]).unwrap();
    let encoded = serde_json::to_string(&p)
        .unwrap()
        .replace("\"schema_version\":1", "\"schema_version\":1.0");
    assert!(Descriptor::parse(encoded.as_bytes()).is_ok());
    let mut value = serde_json::to_value(p).unwrap();
    value["extra"] = serde_json::json!(1);
    assert!(Descriptor::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    value.as_object_mut().unwrap().remove("extra");
    for (field, bad) in [
        ("tool_id", "sample\ntool"),
        ("sha256", "ABC"),
        ("signing_key_id", "key\n"),
        ("signature", "bad"),
        ("version", "01.0.0"),
    ] {
        let mut v = value.clone();
        v[field] = serde_json::json!(bad);
        assert!(
            Descriptor::parse(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{field}"
        );
    }
}
#[test]
fn committed_public_fixture_verifies_without_any_private_key() {
    let p = Descriptor::parse(include_bytes!(
        "../../../tests/fixtures/artifact-signed-test.json"
    ))
    .unwrap();
    let d: SignedKeys = serde_json::from_slice(include_bytes!(
        "../../../shared/fixtures/trust-test-keys.json"
    ))
    .unwrap();
    let r: RootKey = serde_json::from_slice(include_bytes!(
        "../../../shared/fixtures/trust-test-root.json"
    ))
    .unwrap();
    let trust = TrustStore::from_signed(&d, &r, 1, 1000).unwrap();
    let b = include_bytes!("../../../tests/fixtures/artifact-signed-test.tool");
    verify_descriptor(b, &p, &trust, 1000).unwrap();
    let mut corrupted = b.to_vec();
    corrupted[40] ^= 1;
    assert_eq!(
        verify_descriptor(&corrupted, &p, &trust, 1000),
        Err(Error::HashMismatch)
    );
}
