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
    assert_eq!(report["wire_cases"], 33);
    assert_eq!(report["pointer_cases"], 3);
    assert_eq!(report["cpu_interrupted"], true);
    assert_eq!(
        report["sha256"],
        "3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65"
    );
}
