#[path = "support/revocation.rs"]
mod revocation_support;
use kortex_desktop::{ai_bridge, config::Config};
use serde_json::{Value, json};
fn config(dir: &std::path::Path) -> Config {
    Config {
        api_origin: "https://kortex-api-staging.userpgl.workers.dev".into(),
        channel: "staging".into(),
        data_dir: dir.to_path_buf(),
        trust_path: dir.join("trust.json"),
        offline: true,
    }
}
#[test]
fn signed_native_action_uses_universal_runtime_and_keeps_private_marker() {
    let dir = tempfile::tempdir().unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let report: Value = serde_json::from_slice(
        &std::fs::read(
            std::path::PathBuf::from(
                std::env::var("KORTEX_TEST_RELEASES")
                    .unwrap_or_else(|_| root.join("target/phase12").to_string_lossy().into_owned()),
            )
            .join("convertisseur-casse/release-report.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut c = config(dir.path());
    c.trust_path = report["host_trust"].as_str().unwrap().into();
    let folder = dir.path().join("installed/convertisseur-casse/1.0.0");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(
        report["archive"].as_str().unwrap(),
        folder.join("package.tool"),
    )
    .unwrap();
    std::fs::copy(
        report["descriptor"].as_str().unwrap(),
        folder.join("package.tool.json"),
    )
    .unwrap();
    revocation_support::seed(&mut c);
    let marker = dir.path().join("private.txt");
    std::fs::write(&marker, "never upload").unwrap();
    let p = json!({"v":1,"summary":"local","steps":[{"tool_id":"convertisseur-casse","action_id":"upper","input":{"text":"Été ß"},"requires_consent":true}]});
    let now = kortex_runtime_native::permissions::now_ms();
    assert_eq!(
        ai_bridge::execute_reviewed(&c, p.clone(), None, now).unwrap()["text"],
        "ÉTÉ SS"
    );
    let mut unknown = p.clone();
    unknown["steps"][0]["action_id"] = json!("delete-files");
    assert!(ai_bridge::execute_reviewed(&c, unknown, None, now).is_err());
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "never upload");
}
#[test]
fn stale_proposals_and_disconnected_cloud_owners_never_start_a_tool() {
    let dir = tempfile::tempdir().unwrap();
    let c = config(dir.path());
    let plan = json!({"v":1,"summary":"local","steps":[]});
    let now = kortex_runtime_native::permissions::now_ms();
    assert!(
        ai_bridge::execute_reviewed(&c, plan.clone(), None, now - 300000)
            .unwrap_err()
            .contains("expirée")
    );
    assert!(
        ai_bridge::execute_reviewed(&c, plan, Some("signed-out-owner"), now)
            .unwrap_err()
            .contains("Reconnectez")
    );
    assert!(!dir.path().join("installed").exists());
}
#[test]
fn five_signed_tools_execute_locally_without_network_or_file_upload() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut rows = Vec::new();
    for (tool, action, input, field, expected) in [
        (
            "calculatrice",
            "multiply",
            json!({"a":25,"b":48}),
            "result",
            json!(1200.0),
        ),
        (
            "timestamp-utc",
            "to_unix",
            json!({"iso_utc":"2024-01-01T00:00:00Z"}),
            "unix_seconds",
            json!(1704067200),
        ),
        (
            "convertisseur-casse",
            "upper",
            json!({"text":"Été ß"}),
            "text",
            json!("ÉTÉ SS"),
        ),
        (
            "explorateur-csv",
            "summarize_csv",
            json!({"data_text":"a,b\n1,2\n3,4\n"}),
            "rows",
            json!(2),
        ),
        (
            "calculateur-hash",
            "sha256_bytes",
            json!({"data_base64":"YWJj"}),
            "hex",
            json!("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let mut c = config(dir.path());
        let (archive, descriptor, version) = if tool == "calculatrice"
            && std::env::var("KORTEX_TEST_RELEASES").is_err()
        {
            c.trust_path = root.join("target/staging-cloud/host-trust.json");
            (
                root.join("target/staging-cloud/calculatrice-1.1.0.tool"),
                root.join("target/staging-cloud/calculatrice-1.1.0-staging-release-95b59d5e.json"),
                "1.1.0".to_string(),
            )
        } else {
            let report: Value = serde_json::from_slice(
                &std::fs::read(
                    std::path::PathBuf::from(std::env::var("KORTEX_TEST_RELEASES").unwrap_or_else(
                        |_| root.join("target/phase12").to_string_lossy().into_owned(),
                    ))
                    .join(format!("{tool}/release-report.json")),
                )
                .unwrap(),
            )
            .unwrap();
            c.trust_path = report["host_trust"].as_str().unwrap().into();
            (
                std::path::PathBuf::from(report["archive"].as_str().unwrap()),
                std::path::PathBuf::from(report["descriptor"].as_str().unwrap()),
                serde_json::from_slice::<Value>(
                    &std::fs::read(report["descriptor"].as_str().unwrap()).unwrap(),
                )
                .unwrap()["version"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )
        };
        let folder = dir.path().join(format!("installed/{tool}/{version}"));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::copy(archive, folder.join("package.tool")).unwrap();
        std::fs::copy(descriptor, folder.join("package.tool.json")).unwrap();
        revocation_support::seed(&mut c);
        let p = json!({"v":1,"summary":"local validation","steps":[{"tool_id":tool,"action_id":action,"input":input,"requires_consent":true}]});
        let value =
            ai_bridge::execute_reviewed(&c, p, None, kortex_runtime_native::permissions::now_ms())
                .unwrap();
        assert_eq!(value[field], expected, "{tool}");
        rows.push(json!({"tool_id":tool,"action_id":action,"output":value,"offline":true}));
    }
    std::fs::write(
        root.join("target/assistant-native-matrix.json"),
        serde_json::to_vec_pretty(&json!({"passed":true,"real_signed_wasm":true,"rows":rows}))
            .unwrap(),
    )
    .unwrap();
}
