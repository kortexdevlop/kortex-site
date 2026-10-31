use kortex_desktop::{config::Config, revocation, trust};
use kortex_verify::{Descriptor, keys::RootKey};

#[test]
fn production_has_independent_embedded_trust_and_refuses_a_staging_signer() {
    let temp = tempfile::tempdir().unwrap();
    let config = Config {
        api_origin: "https://api.kortex.site".into(),
        channel: "production".into(),
        data_dir: temp.path().into(),
        trust_path: temp.path().join("trust.json"),
        offline: true,
    };
    let root = revocation::root(&config).unwrap();
    let expected: RootKey = serde_json::from_str(include_str!(
        "../../infra/trust/revocation-production-root.json"
    ))
    .unwrap();
    let staging: RootKey =
        serde_json::from_str(include_str!("../../infra/trust/revocation-root.json")).unwrap();
    assert_eq!(root.public_key_base64, expected.public_key_base64);
    assert_ne!(root.public_key_base64, staging.public_key_base64);
    let mut descriptor: Descriptor = serde_json::from_str(include_str!(
        "../../tests/fixtures/artifact-signed-test.json"
    ))
    .unwrap();
    assert!(trust::select(&config, &descriptor).is_err());
    // Selection is separate from artifact signature verification performed on install.
    descriptor.signing_key_id = "production-release-1".into();
    let (selected, _) = trust::select(&config, &descriptor).unwrap();
    assert_eq!(selected.root.public_key_base64, expected.public_key_base64);
}
