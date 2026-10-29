mod common;
use common::*;
use kortex_workflows::{
    runner::{Cancellation, Options, Run},
    *,
};
use serde_json::{Value, json};
#[test]
fn validates_order_mapping_and_empty_queue_has_no_effect() {
    let reg = registry(&[], true, "none");
    let compiled = Compiled::new(&chain(), &reg).unwrap();
    assert_eq!(compiled.order(), vec!["a", "b"]);
    let mut run = Run::new(compiled, Options::default()).unwrap();
    let mut host = TestHost::new(reg);
    assert!(host.calls.is_empty());
    let mut consent = run.consent(0, true);
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut consent,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::Completed
    );
    assert_eq!(run.outputs()["b"]["result"], 146.0);
    assert_eq!(host.calls, vec!["a", "b"]);
    assert_eq!(run.journal().len(), 2);
    assert!(
        !serde_json::to_string(run.journal())
            .unwrap()
            .contains("input")
    );
}
#[test]
fn rejects_cycles_duplicate_nodes_edges_unknown_actions_and_absent_dependencies() {
    let reg = registry(&[], true, "none");
    for kind in [
        "cycle",
        "duplicate",
        "edge",
        "unknown",
        "missing",
        "dependency",
    ] {
        let mut w = workflow();
        match kind {
            "cycle" => w["edges"]
                .as_array_mut()
                .unwrap()
                .push(json!({"from":"b","to":"a"})),
            "duplicate" => w["nodes"][1]["id"] = json!("a"),
            "edge" => w["edges"]
                .as_array_mut()
                .unwrap()
                .push(json!({"from":"a","to":"b"})),
            "unknown" => w["nodes"][0]["action_ref"] = json!("calculatrice.delete-files"),
            "missing" => {
                w["nodes"][0]["input_bindings"]
                    .as_object_mut()
                    .unwrap()
                    .remove("/a");
            }
            _ => {
                w = chain();
                w["edges"] = json!([])
            }
        }
        assert!(Compiled::new(&w, &reg).is_err(), "{kind}");
    }
}
#[test]
fn rejects_type_mismatch_overlapping_bindings_bad_pointers_and_51_nodes() {
    let reg = registry(&[], true, "none");
    for kind in ["type", "overlap", "pointer", "limit"] {
        let mut w = workflow();
        match kind {
            "type" => w["nodes"][0]["input_bindings"]["/a"] = json!({"value":"wrong"}),
            "overlap" => w["nodes"][0]["input_bindings"][""] = json!({"value":{"a":1,"b":2}}),
            "pointer" => w["nodes"][0]["input_bindings"]["/unknown"] = json!({"value":1}),
            _ => {
                w["nodes"] = Value::Array((0..51).map(|i| node(&format!("n{i}"))).collect());
                w["edges"] = json!([])
            }
        }
        assert!(Compiled::new(&w, &reg).is_err(), "{kind}")
    }
}
#[test]
fn runtime_missing_optional_result_stops_before_next_step() {
    let reg = registry(&[], true, "none");
    let mut w = chain();
    w["nodes"][1]["input_bindings"]["/a"]["source_pointer"] = json!("/optional");
    let mut run = Run::new(Compiled::new(&w, &reg).unwrap(), Options::default()).unwrap();
    let mut consent = run.consent(0, true);
    let mut host = TestHost::new(reg);
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut consent,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::MissingInput
    );
    assert_eq!(host.calls, vec!["a"]);
}
#[test]
fn mapping_checks_schema_ranges_escaping_arrays_and_presence() {
    use mapper::*;
    assert_eq!(tokens("/a~1b/~0key").unwrap(), vec!["a/b", "~key"]);
    assert!(tokens("/~2").is_err());
    assert!(!compatible(
        &json!({"type":"number"}),
        &json!({"type":"integer"})
    ));
    assert!(!compatible(
        &json!({"type":"number","minimum":0}),
        &json!({"type":"number","minimum":1})
    ));
    assert!(compatible(
        &json!({"type":"integer","minimum":0,"maximum":10}),
        &json!({"type":"number","minimum":-1,"maximum":20})
    ));
    assert!(
        schema_at(
            &json!({"type":"array","items":{"type":"number"}}),
            &tokens("/0").unwrap()
        )
        .is_ok()
    );
    assert!(
        schema_at(
            &json!({"type":"array","items":{"type":"number"}}),
            &tokens("/01").unwrap()
        )
        .is_err()
    );
}
#[test]
fn retries_only_idempotent_actions_and_stops_invalid_output() {
    let reg = registry(&[], false, "none");
    assert!(matches!(
        Run::new(
            Compiled::new(&workflow(), &reg).unwrap(),
            Options {
                max_retries: 1,
                ..Default::default()
            }
        ),
        Err(Error::RetryForbidden)
    ));
    let reg = registry(&[], true, "none");
    let mut run = Run::new(
        Compiled::new(&workflow(), &reg).unwrap(),
        Options {
            max_retries: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let mut consent = run.consent(0, true);
    let mut host = TestHost::new(reg);
    host.failures = 1;
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut consent,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::Completed
    );
    assert_eq!(host.calls, vec!["a", "a", "b"]);
    let reg = registry(&[], true, "none");
    let mut run = Run::new(
        Compiled::new(&workflow(), &reg).unwrap(),
        Options::default(),
    )
    .unwrap();
    let mut host = TestHost::new(reg);
    host.invalid_output = true;
    let mut c = run.consent(0, true);
    assert_eq!(
        block(run.advance(
            &mut host,
            &mut c,
            &TestClock::default(),
            &Cancellation::default()
        )),
        Error::InvalidOutput
    );
    assert_eq!(host.calls, vec!["a"]);
}

#[test]
fn real_signed_contracts_lock_cross_tool_types_and_unknown_actions() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut reg = Registry::default();
    for (i, id) in ["calculatrice", "timestamp-utc", "convertisseur-casse"]
        .iter()
        .enumerate()
    {
        let (archive, descriptor, trust) = if *id == "calculatrice"
            && std::env::var("KORTEX_TEST_RELEASES").is_err()
        {
            (
                root.join("target/staging-cloud/calculatrice-1.1.0.tool"),
                root.join("target/staging-cloud/calculatrice-1.1.0-staging-release-95b59d5e.json"),
                root.join("target/staging-cloud/host-trust.json"),
            )
        } else {
            let report: Value = serde_json::from_slice(
                &std::fs::read(
                    std::path::PathBuf::from(std::env::var("KORTEX_TEST_RELEASES").unwrap_or_else(
                        |_| root.join("target/phase12").to_string_lossy().into_owned(),
                    ))
                    .join(format!("{id}/release-report.json")),
                )
                .unwrap(),
            )
            .unwrap();
            (
                report["archive"].as_str().unwrap().into(),
                report["descriptor"].as_str().unwrap().into(),
                report["host_trust"].as_str().unwrap().into(),
            )
        };
        let proof = kortex_verify::Descriptor::parse(&std::fs::read(descriptor).unwrap()).unwrap();
        let trust = serde_json::from_slice(&std::fs::read(trust).unwrap()).unwrap();
        let tool = kortex_runtime_native::verify_package_with_trust(
            std::fs::read(archive).unwrap(),
            &proof,
            &trust,
            kortex_runtime_native::permissions::now_ms(),
        )
        .unwrap();
        reg.register_verified(&tool, i as u64 + 1).unwrap();
    }
    let wrong = json!({"workflow_version":"1.0","id":"wrong","name":"Wrong numeric mapping","nodes":[{"id":"calc","action_ref":"calculatrice.multiply","input_bindings":{"/a":{"value":25},"/b":{"value":48}},"requires_confirmation":false},{"id":"date","action_ref":"timestamp-utc.from_unix","input_bindings":{"/value":{"from_node":"calc","source_pointer":"/result"},"/unit":{"value":"seconds"}},"requires_confirmation":false}],"edges":[{"from":"calc","to":"date"}]});
    assert!(matches!(
        Compiled::new(&wrong, &reg),
        Err(Error::TypeMismatch)
    ));
    let valid = json!({"workflow_version":"1.0","id":"valid","name":"Date then number","nodes":[{"id":"date","action_ref":"timestamp-utc.to_unix","input_bindings":{"/iso_utc":{"value":"1970-01-01T00:20:00Z"}},"requires_confirmation":false},{"id":"calc","action_ref":"calculatrice.add","input_bindings":{"/a":{"from_node":"date","source_pointer":"/unix_seconds"},"/b":{"value":10}},"requires_confirmation":false}],"edges":[{"from":"date","to":"calc"}]});
    assert!(Compiled::new(&valid, &reg).is_ok());
    let mut unknown = valid;
    unknown["nodes"][1]["action_ref"] = json!("convertisseur-casse.text_statistics");
    assert!(matches!(
        Compiled::new(&unknown, &reg),
        Err(Error::UnknownAction)
    ));
}
