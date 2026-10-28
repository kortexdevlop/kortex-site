#![cfg(windows)]
use kortex_desktop::{
    account::{self, DeviceSession},
    config::Config,
    library, secure_store,
};
fn fixture() -> (tempfile::TempDir, Config) {
    let dir = tempfile::tempdir().unwrap();
    let c = Config {
        api_origin: "https://kortex-api-staging.userpgl.workers.dev".into(),
        channel: "staging".into(),
        data_dir: dir.path().join("isolated-account"),
        trust_path: dir.path().join("trust.json"),
        offline: true,
    };
    (dir, c)
}
fn token(expires_at: u64) -> DeviceSession {
    DeviceSession {
        access_token: format!("kd_{}", "a".repeat(43)),
        token_type: "Bearer".into(),
        device_id: "14500000-0000-4000-8000-000000000001".into(),
        user_id: "account-test".into(),
        expires_at,
        scopes: vec!["sync:read".into()],
    }
}
struct Clear(Config);
impl Drop for Clear {
    fn drop(&mut self) {
        let _ = secure_store::clear(&self.0);
    }
}
#[test]
fn windows_vault_roundtrip_logout_preserves_tools_and_no_plaintext_disk() {
    let (_dir, c) = fixture();
    let _guard = Clear(c.clone());
    std::fs::create_dir_all(c.data_dir.join("tools")).unwrap();
    let marker = c.data_dir.join("tools/keep.txt");
    std::fs::write(&marker, "private content").unwrap();
    let session = token(u64::MAX);
    secure_store::save(&c, &session).unwrap();
    assert_eq!(
        secure_store::load(&c).unwrap().unwrap().access_token,
        session.access_token
    );
    assert!(account::view(&c).unwrap().connected);
    assert!(!c.data_dir.join("session.json").exists());
    assert_eq!(std::fs::read_dir(&c.data_dir).unwrap().count(), 1);
    assert!(
        account::logout(&c)
            .unwrap()
            .contains("Session locale effacée")
    );
    assert!(secure_store::load(&c).unwrap().is_none());
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "private content");
}
#[test]
fn expired_session_is_removed_and_offline_login_refused() {
    let (_dir, c) = fixture();
    let _guard = Clear(c.clone());
    secure_store::save(&c, &token(1)).unwrap();
    assert!(!account::view(&c).unwrap().connected);
    assert!(secure_store::load(&c).unwrap().is_none());
    assert!(account::start(&c).is_err());
}
#[test]
fn callback_state_duplicate_code_and_path_are_checked() {
    let code = "a".repeat(43);
    assert_eq!(
        account::callback_query(&format!("/callback?code={code}&state=expected"), "expected")
            .unwrap(),
        code
    );
    for target in [
        format!("/callback?code={code}&state=wrong"),
        format!("/callback?code={code}&state=expected&code={code}"),
        format!("/other?code={code}&state=expected"),
        "/callback?code=short&state=expected".into(),
    ] {
        assert!(account::callback_query(&target, "expected").is_err());
    }
}
#[test]
fn remote_references_never_install_or_delete_local_files() {
    let (_dir, c) = fixture();
    std::fs::create_dir_all(&c.data_dir).unwrap();
    let marker = c.data_dir.join("private.txt");
    std::fs::write(&marker, "private").unwrap();
    assert_eq!(
        library::remote_presence(&c, &["calculatrice".into()]).unwrap(),
        vec![("calculatrice".into(), false)]
    );
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "private");
}
