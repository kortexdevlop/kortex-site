#![cfg(windows)]
#[path = "../../crates/kortex-runtime-native/tests/common/mod.rs"]
mod common;
use common::{deadline, run};
use kortex_abi::Envelope;
use kortex_core::{GuestEngine, VerifiedTool};
use kortex_runtime_native::{HostTrust, NativeGuestEngine, verify_package_with_trust};
use serde_json::{Value, json};
use std::{
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[repr(C)]
#[derive(Default)]
struct ProcessMemory {
    cb: u32,
    faults: u32,
    peak_working_set: usize,
    working_set: usize,
    peak_paged: usize,
    paged: usize,
    peak_nonpaged: usize,
    nonpaged: usize,
    pagefile: usize,
    peak_pagefile: usize,
}
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetProcessMemoryInfo(
        process: *mut std::ffi::c_void,
        memory: *mut ProcessMemory,
        size: u32,
    ) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
}
fn rss() -> usize {
    let mut m = ProcessMemory {
        cb: std::mem::size_of::<ProcessMemory>() as u32,
        ..Default::default()
    };
    assert_ne!(
        unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut m,
                std::mem::size_of::<ProcessMemory>() as u32,
            )
        },
        0
    );
    m.working_set
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
fn json_file(p: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap()
}
fn percentile(values: &[f64], fraction: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[(sorted.len() as f64 * fraction).ceil() as usize - 1]
}
fn distribution(values: &[f64]) -> Value {
    json!({"count":values.len(),"p50_ms":percentile(values,0.5),"p95_ms":percentile(values,0.95),"samples_ms":values})
}
fn equivalent(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| equivalent(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, a)| y.get(k).is_some_and(|b| equivalent(a, b)))
        }
        _ => a == b,
    }
}
fn request(kind: &str, payload: Value, n: usize) -> Envelope {
    serde_json::from_value(
        json!({"v":1,"request_id":format!("00000000-0000-4000-8000-{n:012}"),"kind":kind,"payload":payload}),
    )
    .unwrap()
}
fn admit(report: &Value) -> VerifiedTool {
    let bytes = std::fs::read(report["archive"].as_str().unwrap()).unwrap();
    let descriptor = kortex_verify::Descriptor::parse(
        &std::fs::read(report["descriptor"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    let trust: HostTrust =
        serde_json::from_slice(&std::fs::read(report["host_trust"].as_str().unwrap()).unwrap())
            .unwrap();
    verify_package_with_trust(bytes, &descriptor, &trust, now()).unwrap()
}
#[test]
fn five_signed_tools_cold_warm_memory_and_lifecycle() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = std::env::var_os("KORTEX_TEST_RELEASES")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/ci-fixtures"));
    let vectors = json_file(root.join("tests/integration/five-tools-vectors.json"));
    let mut rows = Vec::new();
    let mut engine = NativeGuestEngine;
    for id in [
        "calculatrice",
        "convertisseur-casse",
        "calculateur-hash",
        "timestamp-utc",
        "explorateur-csv",
    ] {
        let release = json_file(fixtures.join(id).join("release-report.json"));
        let vector = vectors
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["tool"] == id)
            .unwrap();
        let mut cold = Vec::new();
        let mut verification = Vec::new();
        let mut instantiation = Vec::new();
        let mut guest_peak = 0;
        for n in 0..10 {
            let start = Instant::now();
            let tool = admit(&release);
            verification.push(start.elapsed().as_secs_f64() * 1000.0);
            let loaded = Instant::now();
            let mut handle = run(engine.load(&tool)).unwrap();
            instantiation.push(loaded.elapsed().as_secs_f64() * 1000.0);
            assert!(
                run(engine.dispatch(&mut handle, &request("init", json!({}), n), deadline()))
                    .unwrap()
                    .ok
            );
            assert!(
                run(engine.dispatch(
                    &mut handle,
                    &request("render", json!({}), n + 10),
                    deadline()
                ))
                .unwrap()
                .ok
            );
            cold.push(start.elapsed().as_secs_f64() * 1000.0);
            guest_peak = guest_peak.max(run(handle.memory_bytes()).unwrap());
            run(engine.unload(handle)).unwrap();
        }
        let tool = admit(&release);
        let mut handle = run(engine.load(&tool)).unwrap();
        run(engine.dispatch(&mut handle, &request("init", json!({}), 30), deadline())).unwrap();
        let mut warm = Vec::new();
        for n in 0..105 {
            let start = Instant::now();
            let result = run(engine.dispatch(
                &mut handle,
                &request(
                    "action.invoke",
                    json!({"action_id":vector["action"],"input":vector["input"]}),
                    100 + n,
                ),
                deadline(),
            ))
            .unwrap();
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            assert!(result.ok);
            assert!(equivalent(
                &serde_json::to_value(result.data).unwrap(),
                &vector["output"]
            ));
            if n >= 5 {
                warm.push(ms)
            }
        }
        guest_peak = guest_peak.max(run(handle.memory_bytes()).unwrap());
        assert!(guest_peak <= 64 * 1024 * 1024);
        run(engine.unload(handle)).unwrap();
        eprintln!(
            "PERF {id}: cold p95 {:.2} ms; warm p95 {:.2} ms",
            percentile(&cold, 0.95),
            percentile(&warm, 0.95)
        );
        rows.push(json!({"tool_id":id,"sha256":release["sha256"],"archive_bytes":std::fs::metadata(release["archive"].as_str().unwrap()).unwrap().len(),"cold_launch":distribution(&cold),"verify":distribution(&verification),"instantiate":distribution(&instantiation),"warm_action":distribution(&warm),"guest_peak_bytes":guest_peak,"thresholds_met":percentile(&cold,0.95)<=3000.0&&percentile(&warm,0.95)<=100.0}));
    }
    let calculator = json_file(fixtures.join("calculatrice/release-report.json"));
    let tool = admit(&calculator);
    let mut eight = Vec::new();
    for _ in 0..8 {
        let h = run(engine.load(&tool)).unwrap();
        assert!(run(h.memory_bytes()).unwrap() <= 64 * 1024 * 1024);
        eight.push(h)
    }
    let eight_rss = rss();
    for h in eight {
        run(engine.unload(h)).unwrap()
    }
    std::thread::sleep(Duration::from_millis(100));
    let baseline = rss();
    let mut memory = Vec::new();
    for n in 0..50 {
        let mut h = run(engine.load(&tool)).unwrap();
        assert!(
            run(engine.dispatch(&mut h, &request("init", json!({}), n + 1000), deadline()))
                .unwrap()
                .ok
        );
        run(engine.unload(h)).unwrap();
        std::thread::sleep(Duration::from_millis(25));
        memory.push(rss())
    }
    std::thread::sleep(Duration::from_millis(100));
    let after = rss();
    let growth = after as f64 / baseline as f64 - 1.0;
    let passed = rows.iter().all(|r| r["thresholds_met"] == true) && growth <= 0.10;
    let report = json!({"date_unix_ms":now(),"platform":"Windows x64 / real Wasmtime / signed local packages","build_profile":if cfg!(debug_assertions){"debug calibration; not release acceptance"}else{"desktop-release inherits release, strip false; same optimization as shipped Desktop"},"cold_definition":"new Engine; package read and verification included; no network; OS filesystem cache not flushed","tools":rows,"eight_instances_rss_bytes":eight_rss,"lifecycle":{"cycles":50,"baseline_rss_bytes":baseline,"after_rss_bytes":after,"growth_fraction":growth,"samples_rss_bytes":memory},"passed":passed});
    let out = std::env::var_os("KORTEX_NATIVE_PERF_REPORT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/ci-reports/runtime-native-performance.json"));
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    std::fs::write(out, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    if std::env::var("KORTEX_PERF_ENFORCE").as_deref() == Ok("1") {
        assert!(
            passed,
            "reference-machine performance budget exceeded; inspect report"
        )
    }
}
