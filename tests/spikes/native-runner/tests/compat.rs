#[test]
fn exact_binary_browser_parity_and_cpu_interruption() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_kortex-native-spike"))
        .output()
        .expect("start native harness");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    assert_eq!(report["wire_cases"], 30);
    assert_eq!(report["pointer_cases"], 3);
    assert_eq!(report["cpu_interrupted"], true);
    assert_eq!(
        report["sha256"],
        "00591ffa2f5faa8398585cd5068058b46efe5409f8ae6011a361844b907793f8"
    );
}
