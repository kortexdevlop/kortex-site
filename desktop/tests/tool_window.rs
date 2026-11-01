mod common;
use common::*;
use kortex_desktop::{launcher, tool_window};
fn args(values: &[&str]) -> impl Iterator<Item = String> {
    let mut all = vec!["kortex-desktop.exe".to_string()];
    all.extend(values.iter().map(|v| v.to_string()));
    all.into_iter()
}
#[test]
fn only_an_exact_tool_request_opens_a_tool_window() {
    assert_eq!(
        tool_window::requested(args(&["--tool", "calculatrice"])).as_deref(),
        Some("calculatrice")
    );
    assert_eq!(tool_window::requested(args(&[])), None);
    assert_eq!(tool_window::requested(args(&["--tool"])), None);
    assert_eq!(tool_window::requested(args(&["--tool", "../evil"])), None);
    assert_eq!(
        tool_window::requested(args(&["--tool", "calculatrice", "--extra"])),
        None
    );
    assert_eq!(tool_window::requested(args(&["calculatrice"])), None);
}
#[test]
fn shortcut_names_stay_inside_the_folder() {
    let folder = std::path::Path::new("bureau");
    assert_eq!(
        tool_window::link_path(folder, "Fusion PDF", "fusion-pdf"),
        folder.join("Fusion PDF.lnk")
    );
    assert_eq!(
        tool_window::link_path(folder, r"..\a/b:c?. ", "fusion-pdf"),
        folder.join("..abc.lnk")
    );
    assert_eq!(
        tool_window::link_path(folder, "...", "fusion-pdf"),
        folder.join("fusion-pdf.lnk")
    );
}
#[test]
fn a_tool_has_one_interactive_session_but_headless_runs_continue() {
    let f = Fixture::new();
    let r = f.release("1.0.0", vec![], false);
    let s = Server::new(r.clone());
    let i = install(&f, &s, &r);
    assert!(!tool_window::focus_open(&f.config, &i.id));
    let first = launcher::prepare_session(&f.config, &s.api, &i.id).unwrap();
    assert!(tool_window::focus_open(&f.config, &i.id));
    assert!(launcher::prepare_session(&f.config, &s.api, &i.id).is_err());
    assert!(launcher::prepare(&f.config, &s.api, &i.id).is_ok());
    drop(first);
    assert!(!tool_window::focus_open(&f.config, &i.id));
    assert!(launcher::prepare_session(&f.config, &s.api, &i.id).is_ok());
}
#[cfg(windows)]
#[test]
fn shortcut_launches_the_tool_and_never_replaces_an_existing_file() {
    let folder = tempfile::tempdir().unwrap();
    let exe = std::env::current_exe().unwrap();
    assert!(
        tool_window::create_shortcut(folder.path(), &exe, "calculatrice", "Calculatrice").unwrap()
    );
    let link = folder.path().join("Calculatrice.lnk");
    let bytes = std::fs::read(&link).unwrap();
    let wide: Vec<u8> = "--tool calculatrice"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    assert!(bytes.windows(wide.len()).any(|w| w == wide));
    assert!(
        !tool_window::create_shortcut(folder.path(), &exe, "calculatrice", "Calculatrice").unwrap()
    );
    assert_eq!(std::fs::read(&link).unwrap(), bytes);
}
