mod common;
use common::*;
use kortex_desktop::{launcher, library};
use sha2::{Digest, Sha256};
#[test]
fn offline_launch_uses_a_verified_package_and_running_lease_blocks_uninstall() {
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    let i = install(&f, &s, &r);
    let plan = launcher::prepare(&f.config, &s.api, &i.id).unwrap();
    assert!(library::uninstall(&f.config, &i.id, false).is_err());
    drop(plan);
    library::uninstall(&f.config, &i.id, false).unwrap();
    assert!(library::scan(&f.config).tools.is_empty());
}
#[test]
fn uninstall_preserves_data_by_default_and_purges_only_the_current_profile_on_request() {
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    let i = install(&f, &s, &r);
    let profile = library::profile(&f.config, &i.id).unwrap();
    std::fs::write(&profile, b"local settings").unwrap();
    let other = profile
        .parent()
        .unwrap()
        .join(format!("{:x}.json", Sha256::digest(b"other-profile")));
    std::fs::write(&other, b"other profile").unwrap();
    let other_tool = f.config.data_dir.join("tools/another-tool");
    std::fs::create_dir_all(&other_tool).unwrap();
    std::fs::write(other_tool.join("keep"), b"other tool").unwrap();
    library::uninstall(&f.config, &i.id, false).unwrap();
    assert_eq!(std::fs::read(&profile).unwrap(), b"local settings");
    install(&f, &s, &r);
    library::uninstall(&f.config, &i.id, true).unwrap();
    assert!(!profile.exists());
    assert_eq!(std::fs::read(other).unwrap(), b"other profile");
    assert!(other_tool.join("keep").exists());
    assert!(library::load(&f.config).unwrap().tools.is_empty());
}
#[test]
fn confirmed_online_revocation_is_remembered_offline_and_missing_managed_index_fails_closed() {
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    let i = install(&f, &s, &r);
    let mut online = f.config.clone();
    online.offline = false;
    s.mode("revoked");
    assert!(launcher::prepare(&online, &s.api, &i.id).is_err());
    assert!(launcher::prepare(&f.config, &s.api, &i.id).is_err());
    std::fs::remove_file(f.config.data_dir.join("library.json")).unwrap();
    assert!(library::scan(&f.config).tools.is_empty());
    assert!(library::load(&f.config).is_err());
}
