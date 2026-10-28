mod common;
use common::*;
use kortex_desktop::{launcher, library, updates};
use std::sync::atomic::AtomicBool;
#[test]
fn semver_permissions_and_successful_update_retain_old_version_until_confirmed_start() {
    let f = Fixture::new();
    let old = f.release("1.0.0", vec![], false);
    let s = Server::new(old.clone());
    let i = install(&f, &s, &old);
    assert!(updates::check(&f.config, &s.api).unwrap().is_empty());
    let next = f.release("1.1.0", vec!["file.read".into()], false);
    s.publish(next.clone());
    let u = updates::check(&f.config, &s.api).unwrap().remove(0);
    assert_eq!(u.extra_permissions, ["file.read"]);
    assert!(
        updates::apply_checked(
            &f.config,
            &s.api,
            &u,
            &[],
            &AtomicBool::new(false),
            &monitor(),
            |_| Ok(())
        )
        .is_err()
    );
    let p = library::profile(&f.config, &i.id).unwrap();
    std::fs::write(&p, b"settings before update").unwrap();
    let installed = updates::apply(
        &f.config,
        &s.api,
        &u,
        &next.detail.manifest.permissions,
        &AtomicBool::new(false),
        &monitor(),
    )
    .unwrap();
    assert_eq!(installed.version, "1.1.0");
    assert_eq!(std::fs::read(&p).unwrap(), b"settings before update");
    let index = library::load(&f.config).unwrap();
    assert_eq!(index.tools[&i.id].versions.len(), 2);
    assert!(index.tools[&i.id].pending.is_some());
    updates::confirm_launch(&f.config, &i.id, "1.1.0").unwrap();
    assert!(
        library::load(&f.config).unwrap().tools[&i.id]
            .pending
            .is_none()
    );
    assert_eq!(
        updates::rollback(&f.config, &i.id).unwrap().version,
        "1.0.0"
    );
}
#[test]
fn real_guest_start_failure_and_migration_failure_leave_old_version_and_data_intact() {
    let f = Fixture::new();
    let old = f.release("1.0.0", vec![], false);
    let s = Server::new(old.clone());
    let i = install(&f, &s, &old);
    let p = library::profile(&f.config, &i.id).unwrap();
    std::fs::write(&p, b"original settings").unwrap();
    s.publish(f.release("1.1.0", vec![], true));
    let u = updates::check(&f.config, &s.api).unwrap().remove(0);
    assert!(
        updates::apply(
            &f.config,
            &s.api,
            &u,
            &[],
            &AtomicBool::new(false),
            &monitor()
        )
        .is_err()
    );
    assert_eq!(library::current(&f.config, &i.id).unwrap().version, "1.0.0");
    assert_eq!(std::fs::read(&p).unwrap(), b"original settings");
    s.publish(f.release("1.2.0", vec![], false));
    let u = updates::check(&f.config, &s.api).unwrap().remove(0);
    assert!(
        updates::apply_checked(
            &f.config,
            &s.api,
            &u,
            &[],
            &AtomicBool::new(false),
            &monitor(),
            |_| {
                std::fs::write(&p, b"partly migrated").unwrap();
                Err("migration rejected".into())
            }
        )
        .is_err()
    );
    assert_eq!(std::fs::read(&p).unwrap(), b"original settings");
    assert_eq!(library::current(&f.config, &i.id).unwrap().version, "1.0.0");
}
#[test]
fn unconfirmed_first_launch_rollback_restores_saved_data_and_blocks_failed_version() {
    let f = Fixture::new();
    let old = f.release("1.0.0", vec![], false);
    let s = Server::new(old.clone());
    let i = install(&f, &s, &old);
    let profile = library::profile(&f.config, &i.id).unwrap();
    std::fs::write(&profile, b"old profile").unwrap();
    s.publish(f.release("1.1.0", vec![], false));
    let u = updates::check(&f.config, &s.api).unwrap().remove(0);
    updates::apply_checked(
        &f.config,
        &s.api,
        &u,
        &[],
        &AtomicBool::new(false),
        &monitor(),
        |_| Ok(()),
    )
    .unwrap();
    std::fs::write(&profile, b"new profile").unwrap();
    let result = updates::rollback(&f.config, &i.id).unwrap();
    assert_eq!(result.version, "1.0.0");
    assert_eq!(std::fs::read(profile).unwrap(), b"old profile");
    assert!(library::load(&f.config).unwrap().tools[&i.id].versions["1.1.0"].blocked);
    assert!(launcher::prepare(&f.config, &s.api, &i.id).is_ok());
}

#[test]
fn cancel_before_activation_preserves_old_current_and_profile() {
    let f = Fixture::new();
    let old = f.release("1.0.0", vec![], false);
    let s = Server::new(old.clone());
    let i = install(&f, &s, &old);
    s.publish(f.release("1.1.0", vec![], false));
    let u = updates::check(&f.config, &s.api).unwrap().remove(0);
    let cancel = AtomicBool::new(false);
    assert!(
        updates::apply_checked(&f.config, &s.api, &u, &[], &cancel, &monitor(), |_| {
            cancel.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        })
        .is_err()
    );
    assert_eq!(library::current(&f.config, &i.id).unwrap().version, "1.0.0");
}

#[test]
fn semver_precedence_ignores_build_metadata_and_checks_minimum_runtime() {
    assert!(!kortex_desktop::versions::newer("1.0.0+new", "1.0.0+old").unwrap());
    assert!(kortex_desktop::versions::newer("1.1.0", "1.0.9").unwrap());
    assert!(!kortex_desktop::versions::newer("1.1.0-beta", "1.1.0").unwrap());
    assert!(kortex_desktop::versions::supported("1.0.0+build").unwrap());
    assert!(!kortex_desktop::versions::supported("9.0.0").unwrap());
}
