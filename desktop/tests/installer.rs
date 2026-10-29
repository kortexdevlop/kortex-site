mod common;
use common::*;
use kortex_desktop::{installer, library, paths};
use std::sync::atomic::AtomicBool;
#[test]
fn signed_install_is_atomic_idempotent_and_never_extracts_guest_paths() {
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    let i = install(&f, &s, &r);
    assert_eq!(
        library::current(&f.config, &i.id).unwrap().sha256,
        r.detail.descriptor.sha256
    );
    let revision = library::load(&f.config).unwrap().revision;
    install(&f, &s, &r);
    assert_eq!(library::load(&f.config).unwrap().revision, revision);
    assert_eq!(
        std::fs::read_dir(
            f.config
                .data_dir
                .join("installed")
                .join(&i.id)
                .join(&i.version)
        )
        .unwrap()
        .count(),
        2
    );
    assert_eq!(
        std::fs::read_dir(f.config.data_dir.join("staging"))
            .unwrap()
            .count(),
        0
    );
    let external = f.temp.path().join("outside");
    std::fs::create_dir(&external).unwrap();
    assert!(paths::remove_tree(&f.config.data_dir, &external).is_err());
    assert!(external.exists());
}
#[test]
fn interrupted_tampered_oversized_redirect_and_cancelled_downloads_leave_no_active_package() {
    for mode in ["truncated", "tampered", "oversized", "redirect"] {
        let f = Fixture::new();
        let r = f.release("1.0.0", vec![], false);
        let s = Server::new(r.clone());
        s.mode(mode);
        let rejected = installer::install(
            &f.config,
            &s.api,
            &r.detail,
            &[],
            &AtomicBool::new(false),
            &monitor(),
            true,
        )
        .unwrap_err();
        assert!(
            !rejected.contains("Hors connexion"),
            "fixture transport failed before {mode}: {rejected}"
        );
        assert!(library::scan(&f.config).tools.is_empty());
        assert_eq!(
            std::fs::read_dir(f.config.data_dir.join("staging"))
                .unwrap()
                .count(),
            0
        );
        s.mode("normal");
        assert_eq!(
            installer::install(
                &f.config,
                &s.api,
                &r.detail,
                &[],
                &AtomicBool::new(false),
                &monitor(),
                true
            )
            .unwrap_or_else(|e| panic!("retry {mode}: {e}"))
            .version,
            "1.0.0"
        );
    }
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    assert!(
        installer::install(
            &f.config,
            &s.api,
            &r.detail,
            &[],
            &AtomicBool::new(true),
            &monitor(),
            true
        )
        .is_err()
    );
    assert!(library::scan(&f.config).tools.is_empty());
}
#[test]
fn invalid_signature_unconfirmed_permissions_and_unwritable_storage_are_refused() {
    let f = Fixture::new();
    let mut r = f.release("1.0.0", vec!["file.read".into()], false);
    let s = Server::new(r.clone());
    assert!(
        installer::install(
            &f.config,
            &s.api,
            &r.detail,
            &[],
            &AtomicBool::new(false),
            &monitor(),
            true
        )
        .is_err()
    );
    r.detail.descriptor.signature =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [0u8; 64]);
    s.publish(r.clone());
    assert!(
        installer::install(
            &f.config,
            &s.api,
            &r.detail,
            &r.detail.manifest.permissions,
            &AtomicBool::new(false),
            &monitor(),
            true
        )
        .is_err()
    );
    assert!(library::scan(&f.config).tools.is_empty());
    let mut f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    f.config.data_dir = f.temp.path().join("blocked-storage");
    std::fs::write(&f.config.data_dir, b"not a directory").unwrap();
    assert!(
        installer::install(
            &f.config,
            &s.api,
            &r.detail,
            &[],
            &AtomicBool::new(false),
            &monitor(),
            true
        )
        .is_err()
    );
}
#[test]
fn signed_zip_traversal_is_rejected_without_writing_outside_the_install() {
    use std::io::Write;
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file("../escaped", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"hostile").unwrap();
    let bytes = zip.finish().unwrap().into_inner();
    let r = f.signed(
        bytes,
        serde_json::to_value(r.detail.manifest).unwrap(),
        serde_json::to_value(r.detail.contract).unwrap(),
    );
    let s = Server::new(r.clone());
    assert!(
        installer::install(
            &f.config,
            &s.api,
            &r.detail,
            &[],
            &AtomicBool::new(false),
            &monitor(),
            true
        )
        .is_err()
    );
    assert!(!f.temp.path().join("escaped").exists());
    assert!(library::scan(&f.config).tools.is_empty());
}

#[test]
fn cancellation_during_streaming_removes_partial_download() {
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    s.mode("slow");
    let cancel = std::sync::Arc::new(AtomicBool::new(false));
    let progress = monitor();
    let token = cancel.clone();
    let m = progress.clone();
    let c = f.config.clone();
    let api = s.api.clone();
    let d = r.detail.clone();
    let task = std::thread::spawn(move || installer::install(&c, &api, &d, &[], &token, &m, true));
    let started = std::time::Instant::now();
    while progress.lock().unwrap().downloaded == 0 {
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(task.join().unwrap().is_err());
    assert!(library::scan(&f.config).tools.is_empty());
    assert_eq!(
        std::fs::read_dir(f.config.data_dir.join("staging"))
            .unwrap()
            .count(),
        0
    );
}
