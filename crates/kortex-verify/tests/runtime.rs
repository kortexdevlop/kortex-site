use kortex_verify::{keys::RootKey, runtime::Envelope};
use serde_json::Value;
#[test]
fn runtime_metadata_matches_web_golden_and_refuses_tamper_replay_and_expiry() {
    let f: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/runtime-channels.signed-test.json"
    ))
    .unwrap();
    let root: RootKey = serde_json::from_value(f["root"].clone()).unwrap();
    let mut e: Envelope = serde_json::from_value(f["envelope"].clone()).unwrap();
    let now = f["now"].as_u64().unwrap();
    assert_eq!(
        String::from_utf8(e.document.signing_text().unwrap()).unwrap(),
        f["signing_text"].as_str().unwrap()
    );
    assert!(e.verify(&root, 7, now).is_ok());
    assert!(e.verify(&root, 8, now).is_err());
    assert!(e.verify(&root, 7, now + 3600000).is_err());
    e.document.sequence = 8;
    assert!(e.verify(&root, 7, now).is_err());
}
