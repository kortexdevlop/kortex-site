mod common;
use common::Fixture;
use kortex_capabilities::storage::{Owner, ToolStorage};
use kortex_desktop::{config::Config, workflow_permissions::Permissions, workflows};
use kortex_workflows::runner::Cancellation;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
fn package(f: &Fixture, id: &str) -> (Vec<u8>, Value) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let wasm=std::fs::read(root.join("target/workflow-capability-guest/wasm32-unknown-unknown/release/kortex_workflow_capability_test.wasm")).unwrap();
    let mut files = f.base.clone();
    let mut manifest: Value = serde_json::from_slice(&files["manifest.json"]).unwrap();
    manifest["id"] = json!(id);
    manifest["version"] = json!("1.0.0");
    manifest["name"] = json!("Test des autorisations");
    manifest["permissions"] = json!(["storage.read", "storage.write"]);
    let actions=["write","read","escape","double"].map(|id|json!({"id":id,"description":format!("Action {id}"),"input_schema":{"type":"object","properties":{"key":{"type":"string","maxLength":256},"text":{"type":"string","maxLength":4096}},"required":if id=="read"{vec!["key"]}else{vec!["key","text"]},"additionalProperties":false},"output_schema":{"type":"object","properties":{"result":{"type":"string","maxLength":4096}},"required":["result"],"additionalProperties":false},"permissions":[if id=="read"{"storage.read"}else{"storage.write"}],"side_effect":if id=="read"{"none"}else{"local-write"},"idempotent":false,"timeout_ms":10000}));
    let contract = json!({"contract_version":"1.0","tool_id":id,"actions":actions});
    files.insert(
        "manifest.json".into(),
        serde_json::to_vec(&manifest).unwrap(),
    );
    files.insert(
        "contract.json".into(),
        serde_json::to_vec(&contract).unwrap(),
    );
    files.insert("module.wasm".into(), wasm);
    let release = f.signed(kortex_pack::package(files).unwrap(), manifest, contract);
    (
        release.bytes,
        serde_json::to_value(release.detail.descriptor).unwrap(),
    )
}
fn install(f: &Fixture, c: &Config, id: &str) {
    let (bytes, proof) = package(f, id);
    let dir = c.data_dir.join("installed").join(id).join("1.0.0");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.tool"), bytes).unwrap();
    std::fs::write(
        dir.join("package.tool.json"),
        serde_json::to_vec(&proof).unwrap(),
    )
    .unwrap();
}
fn value(c: &Config, id: &str, key: &str) -> Option<Value> {
    let mut backend =
        kortex_runtime_native::storage::NativeStorage::new(c.data_dir.join("data")).unwrap();
    ToolStorage::new(
        &mut backend,
        Owner {
            profile: "local".into(),
            tool_id: id.into(),
        },
        1,
    )
    .unwrap()
    .get(key)
    .unwrap()
    .map(|b| serde_json::from_slice(&b).unwrap())
}
fn node(id: &str, tool: &str, action: &str, key: &str) -> Value {
    json!({"id":id,"action_ref":format!("{tool}.{action}"),"input_bindings":{"/key":{"value":key},"/text":{"value":"private-test-value"}},"requires_confirmation":true})
}
fn workflow(nodes: Vec<Value>) -> Value {
    json!({"workflow_version":"1.0","id":"sensitive","name":"Accès locaux","nodes":nodes,"edges":[]})
}
fn execute(
    c: &Config,
    w: Value,
    answer: impl Fn(usize, &kortex_desktop::workflow_permissions::Prompt, &Permissions, &Cancellation)
    + Send
    + 'static,
) -> (Value, usize) {
    let broker = Permissions::interactive();
    let cancel = Cancellation::default();
    let child = broker.clone();
    let token = cancel.clone();
    let config = c.clone();
    let count = Arc::new(AtomicUsize::new(0));
    let seen = count.clone();
    let work = std::thread::spawn(move || {
        workflows::execute_with_permissions(
            &config,
            w,
            true,
            token,
            Arc::new(Mutex::new(vec![])),
            child,
        )
    });
    let mut ticket = String::new();
    let end = std::time::Instant::now() + Duration::from_secs(60);
    while !work.is_finished() {
        assert!(std::time::Instant::now() < end, "permission test timed out");
        if let Some(p) = broker.pending() {
            if p.id != ticket {
                ticket = p.id.clone();
                let n = seen.fetch_add(1, Ordering::SeqCst);
                answer(n, &p, &broker, &cancel)
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let result = work.join().unwrap().unwrap();
    (result, count.load(Ordering::SeqCst))
}
#[test]
fn signed_sensitive_workflows_use_real_gateway_and_exact_single_use_consent() {
    let f = Fixture::new();
    let id = "workflow-sensitive-test";
    let c = f.config.clone();
    install(&f, &c, id);
    let mut read = node("read", id, "read", "approved");
    read["input_bindings"]
        .as_object_mut()
        .unwrap()
        .remove("/text");
    let (result, count) = execute(
        &c,
        workflow(vec![node("write", id, "write", "approved"), read]),
        |_, p, b, _| b.respond(&p.id, true, p.resource.clone()),
    );
    assert_eq!(result["status"], "Completed");
    assert_eq!(result["outputs"]["read"]["result"], "private-test-value");
    assert_eq!(count, 2);
    assert_eq!(value(&c, id, "approved"), Some(json!("private-test-value")));
    let (result, _) = execute(
        &c,
        workflow(vec![node("deny", id, "write", "denied")]),
        |_, p, b, _| b.respond(&p.id, false, p.resource.clone()),
    );
    assert_eq!(result["status"], "PermissionDenied");
    assert_eq!(value(&c, id, "denied"), None);
    let (result, _) = execute(
        &c,
        workflow(vec![node("escape", id, "escape", "approved")]),
        |_, p, b, _| b.respond(&p.id, true, p.resource.clone()),
    );
    assert_eq!(result["status"], "ActionFailed");
    assert_eq!(value(&c, id, "unapproved"), None);
    let (result, _) = execute(
        &c,
        workflow(vec![node("once", id, "double", "once")]),
        |_, p, b, _| b.respond(&p.id, true, p.resource.clone()),
    );
    assert_eq!(result["status"], "ActionFailed");
    assert_eq!(value(&c, id, "once"), Some(json!("private-test-value")));
    let (result, _) = execute(
        &c,
        workflow(vec![
            node("first", id, "write", "before-cancel"),
            node("second", id, "write", "after-cancel"),
        ]),
        |n, p, b, c| {
            if n == 0 {
                b.respond(&p.id, true, p.resource.clone())
            } else {
                c.cancel()
            }
        },
    );
    assert_eq!(result["status"], "Cancelled");
    assert_eq!(value(&c, id, "after-cancel"), None);
    assert!(result["outputs"]["first"].is_object());
    let other = "workflow-sensitive-other";
    install(&f, &c, other);
    let mut read = node("other", other, "read", "approved");
    read["input_bindings"]
        .as_object_mut()
        .unwrap()
        .remove("/text");
    let (result, _) = execute(&c, workflow(vec![read]), |_, p, b, _| {
        b.respond(&p.id, true, p.resource.clone())
    });
    assert_eq!(result["outputs"]["other"]["result"], "");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let dir = root.join("target/workflow-sensitive-fixture");
    let (bytes, proof) = package(&f, id);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.tool"), bytes).unwrap();
    std::fs::write(
        dir.join("package.tool.json"),
        serde_json::to_vec(&proof).unwrap(),
    )
    .unwrap();
    std::fs::copy(&c.trust_path, dir.join("trust.json")).unwrap();
    std::fs::write(root.join("target/workflow-sensitive-proof.json"),serde_json::to_vec_pretty(&json!({"passed":true,"real_signed_wasm":true,"real_native_services_gateway":true,"approval_then_read":true,"deny_has_no_write":true,"scope_escape_refused":true,"single_use_grant":true,"cancel_stops_next_node":true,"separate_tool_namespaces":true,"cloud_execution":false})).unwrap()).unwrap();
}
