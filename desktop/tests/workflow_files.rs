use kortex_desktop::{config::Config, workflows};
use kortex_workflows::runner::Cancellation;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
#[test]
fn real_signed_native_file_hash_needs_consent_and_never_changes_saved_definition() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let report: Value = serde_json::from_slice(
        &std::fs::read(root.join("target/phase12/calculateur-hash/release-report.json")).unwrap(),
    )
    .unwrap();
    let temp = tempfile::tempdir().unwrap();
    let installed = temp.path().join("installed/calculateur-hash/1.0.0");
    std::fs::create_dir_all(&installed).unwrap();
    std::fs::copy(
        report["archive"].as_str().unwrap(),
        installed.join("package.tool"),
    )
    .unwrap();
    std::fs::copy(
        report["descriptor"].as_str().unwrap(),
        installed.join("package.tool.json"),
    )
    .unwrap();
    let c = Config {
        api_origin: "https://kortex-api-staging.userpgl.workers.dev".into(),
        channel: "staging".into(),
        data_dir: temp.path().into(),
        trust_path: report["host_trust"].as_str().unwrap().into(),
        offline: true,
    };
    let file = temp.path().join("private.txt");
    std::fs::write(&file, b"abc").unwrap();
    let original = json!({"workflow_version":"1.0","id":"private_hash","name":"Hash local","nodes":[{"id":"hash","action_ref":"calculateur-hash.sha256_bytes","input_bindings":{"/data_base64":{"value":""}},"requires_confirmation":true}],"edges":[]});
    let cancel = Cancellation::default();
    assert!(workflows::bind_file(&original, "hash", &file, false, &cancel).is_err());
    assert!(
        workflows::execute(
            &c,
            original.clone(),
            false,
            cancel.clone(),
            Arc::new(Mutex::new(vec![]))
        )
        .is_err()
    );
    let bound = workflows::bind_file(&original, "hash", &file, true, &cancel).unwrap();
    let result = workflows::execute(&c, bound, true, cancel, Arc::new(Mutex::new(vec![]))).unwrap();
    assert_eq!(result["status"], "Completed");
    assert_eq!(
        result["outputs"]["hash"]["hex"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        original["nodes"][0]["input_bindings"]["/data_base64"]["value"],
        ""
    );
    std::fs::write(root.join("target/orchestration-native-file.json"),serde_json::to_vec_pretty(&json!({"passed":true,"real_signed_wasm":true,"offline":true,"no_file_binding_without_consent":true,"definition_unchanged":true,"sha256_abc":result["outputs"]["hash"]["hex"]})).unwrap()).unwrap();
}
