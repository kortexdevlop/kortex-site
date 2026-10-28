use kortex_desktop::{
    config::{Config, validate_origin},
    library,
};
use std::path::Path;
#[test]
fn config_pins_https_origins_and_library_starts_offline_without_account() {
    for bad in [
        "http://api.kortex.site",
        "https://api.kortex.site@evil.example",
        "https://evil.example",
        "https://api.kortex.site/path",
        "https://api.kortex.site?x=1",
        "https://api.kortex.site:8443",
    ] {
        assert!(validate_origin(bad).is_err(), "{bad}");
    }
    assert_eq!(
        validate_origin("https://kortex-api-staging.userpgl.workers.dev").unwrap(),
        "staging"
    );
    let temp = tempfile::tempdir().unwrap();
    let c = Config {
        api_origin: "https://api.kortex.site".into(),
        channel: "production".into(),
        data_dir: temp.path().into(),
        trust_path: temp.path().join("trust.json"),
        offline: true,
    };
    assert!(library::scan(&c).tools.is_empty());
    assert!(!temp.path().join("installed").exists());
}
#[test]
fn installed_signed_package_remains_available_offline_and_tampering_is_refused() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let temp = tempfile::tempdir().unwrap();
    let release: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("target/phase12/convertisseur-casse/release-report.json"))
            .unwrap(),
    )
    .unwrap();
    let folder = temp.path().join("installed/convertisseur-casse/1.0.0");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(
        release["archive"].as_str().unwrap(),
        folder.join("package.tool"),
    )
    .unwrap();
    std::fs::copy(
        release["descriptor"].as_str().unwrap(),
        folder.join("package.tool.json"),
    )
    .unwrap();
    let c = Config {
        api_origin: "https://api.kortex.site".into(),
        channel: "production".into(),
        data_dir: temp.path().into(),
        trust_path: release["host_trust"].as_str().unwrap().into(),
        offline: true,
    };
    let list = library::scan(&c);
    assert_eq!(list.tools.len(), 1);
    assert_eq!(list.tools[0].name, "Convertisseur de casse");
    assert_eq!(list.rejected, 0);
    let mut bytes = std::fs::read(folder.join("package.tool")).unwrap();
    bytes[0] ^= 1;
    std::fs::write(folder.join("package.tool"), bytes).unwrap();
    let refused = library::scan(&c);
    assert!(refused.tools.is_empty());
    assert_eq!(refused.rejected, 1);
}
