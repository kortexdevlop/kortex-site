mod common;
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use kortex_desktop::{
    api::Api,
    revocation::{self, Cache},
};
#[test]
fn offline_launch_checks_signed_cache_expiry_clock_and_permanent_revocation() {
    let f = common::Fixture::new();
    let p = f.release("1.0.0", vec![], false).detail.descriptor;
    let api = Api::new(&f.config).unwrap();
    let root = revocation::root(&f.config).unwrap();
    let path = revocation::cache_path(&f.config, &root);
    assert!(revocation::check(&f.config, &api, &p).is_ok());
    let now = kortex_runtime_native::permissions::now_ms();
    let mut d = common::revocation_document(now);
    let save = |d: &kortex_verify::revocation::Document, seen| {
        std::fs::write(
            &path,
            serde_json::to_vec(&Cache {
                document: d.clone(),
                seen,
            })
            .unwrap(),
        )
        .unwrap()
    };
    save(&d, now + 60000);
    assert!(
        revocation::check(&f.config, &api, &p)
            .unwrap_err()
            .contains("Horloge")
    );
    d.issued_at = now - 86401000;
    d.valid_until = now - 1000;
    d.signature = STANDARD.encode(
        SigningKey::from_bytes(&[43; 32])
            .sign(&d.signing_text().unwrap())
            .to_bytes(),
    );
    save(&d, now);
    assert!(
        revocation::check(&f.config, &api, &p)
            .unwrap_err()
            .contains("24 heures")
    );
    d.revoked_keys.push(p.signing_key_id.clone());
    d.signature = STANDARD.encode(
        SigningKey::from_bytes(&[43; 32])
            .sign(&d.signing_text().unwrap())
            .to_bytes(),
    );
    save(&d, now);
    assert_eq!(
        revocation::check(&f.config, &api, &p).unwrap_err(),
        "REVOKED"
    );
    d.revoked_keys.clear();
    save(&d, now);
    assert!(
        revocation::check(&f.config, &api, &p)
            .unwrap_err()
            .contains("Signature")
    );
}
