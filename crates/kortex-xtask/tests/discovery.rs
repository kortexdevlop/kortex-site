use kortex_xtask::discovery::discover;
use serde_json::json;
fn add(root: &std::path::Path, folder: &str, id: &str) {
    let dir = root.join(folder);
    std::fs::create_dir_all(&dir).unwrap();
    let mut m: serde_json::Value =
        serde_json::from_str(include_str!("../../../tools/calculatrice/manifest.json")).unwrap();
    m["id"] = json!(id);
    std::fs::write(dir.join("manifest.json"), serde_json::to_vec(&m).unwrap()).unwrap();
}
#[test]
fn arbitrary_tools_are_discovered_without_identifier_branches() {
    let temp = tempfile::tempdir().unwrap();
    add(temp.path(), "unrelated-directory", "sixth-example");
    add(temp.path(), "another-folder", "other-example");
    let found = discover(temp.path()).unwrap();
    assert_eq!(
        found.keys().map(String::as_str).collect::<Vec<_>>(),
        ["other-example", "sixth-example"]
    );
    assert!(
        found["sixth-example"]
            .directory
            .ends_with("unrelated-directory")
    );
}
#[test]
fn duplicate_ids_and_invalid_manifests_are_refused() {
    let temp = tempfile::tempdir().unwrap();
    add(temp.path(), "one", "sixth-example");
    add(temp.path(), "two", "sixth-example");
    assert_eq!(discover(temp.path()).err().unwrap(), "DUPLICATE_TOOL_ID");
    std::fs::write(temp.path().join("two/manifest.json"), b"{}").unwrap();
    assert_eq!(discover(temp.path()).err().unwrap(), "INVALID_MANIFEST");
}
#[test]
fn cli_help_and_bad_arguments() {
    use std::process::Command;
    assert!(
        Command::new(env!("CARGO_BIN_EXE_kortex-xtask"))
            .arg("--help")
            .output()
            .unwrap()
            .status
            .success()
    );
    for args in [
        vec!["build"],
        vec!["build", "--all", "--tool", "calculatrice"],
        vec!["verify", "--tool", "calculatrice"],
        vec!["build", "--bogus"],
    ] {
        assert!(
            !Command::new(env!("CARGO_BIN_EXE_kortex-xtask"))
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    assert!(
        !Command::new(env!("CARGO_BIN_EXE_kortex-xtask"))
            .env_remove("CI")
            .args(["sign", "--artifact", "missing.tool"])
            .output()
            .unwrap()
            .status
            .success()
    );
}
