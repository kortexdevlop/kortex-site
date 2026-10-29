//! Test adapter only: proves the generic runner uses the real universal runtime and signed WASM.
use kortex_capabilities::policy::Policy;
use kortex_core::{Runtime, dispatcher::Clock};
use kortex_workflows::{
    permissions::Scope,
    runner::{Cancellation, Host, InvokeError, Options, Run},
    *,
};
use serde_json::{Value, json};
struct NativeClock;
impl Clock for NativeClock {
    fn now_ms(&self) -> u64 {
        kortex_runtime_native::permissions::now_ms()
    }
    async fn wait_until(&self, deadline: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(
            deadline.saturating_sub(self.now_ms()),
        ))
        .await
    }
}
struct Adapter {
    registry: Registry,
    policy: Policy,
    runtime: Runtime<kortex_runtime_native::NativeGuestEngine, NativeClock>,
}
impl Host for Adapter {
    fn registry(&self) -> &Registry {
        &self.registry
    }
    fn policy(&self) -> &Policy {
        &self.policy
    }
    fn scopes(&self, _: &Node, s: &ActionSpec, _: &Value) -> Result<Vec<Scope>, Error> {
        if !s.action.permissions.is_empty() {
            return Err(Error::PermissionDenied);
        }
        Ok(vec![])
    }
    async fn invoke(
        &mut self,
        _: &Node,
        s: &ActionSpec,
        input: Value,
        _: u64,
    ) -> Result<Value, InvokeError> {
        let reply = self
            .runtime
            .invoke_action(s.context.instance_id, &s.action.id, input)
            .await
            .map_err(|_| InvokeError { retryable: false })?;
        if !reply.ok {
            return Err(InvokeError { retryable: false });
        }
        reply
            .data
            .map(Value::Object)
            .ok_or(InvokeError { retryable: false })
    }
}
#[test]
fn real_signed_two_step_workflow_runs_locally_without_cloud_execution() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (archive_path, descriptor_path, trust_path) =
        if let Ok(fixtures) = std::env::var("KORTEX_TEST_RELEASES") {
            let report: Value = serde_json::from_slice(
                &std::fs::read(
                    std::path::Path::new(&fixtures).join("calculatrice/release-report.json"),
                )
                .unwrap(),
            )
            .unwrap();
            (
                std::path::PathBuf::from(report["archive"].as_str().unwrap()),
                std::path::PathBuf::from(report["descriptor"].as_str().unwrap()),
                std::path::PathBuf::from(report["host_trust"].as_str().unwrap()),
            )
        } else {
            (
                root.join("target/staging-cloud/calculatrice-1.1.0.tool"),
                root.join("target/staging-cloud/calculatrice-1.1.0-staging-release-95b59d5e.json"),
                root.join("target/staging-cloud/host-trust.json"),
            )
        };
    let archive = std::fs::read(archive_path).unwrap();
    let descriptor =
        kortex_verify::Descriptor::parse(&std::fs::read(descriptor_path).unwrap()).unwrap();
    let trust = serde_json::from_slice(&std::fs::read(trust_path).unwrap()).unwrap();
    let tool = kortex_runtime_native::verify_package_with_trust(
        archive,
        &descriptor,
        &trust,
        kortex_runtime_native::permissions::now_ms(),
    )
    .unwrap();
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let proof=executor.block_on(async{let mut runtime=Runtime::new(kortex_runtime_native::NativeGuestEngine,NativeClock);let id=runtime.load(tool.clone()).await.unwrap();runtime.initialize(id).await.unwrap();let mut registry=Registry::default();registry.register_verified(&tool,id).unwrap();let workflow=json!({"workflow_version":"1.0","id":"native_proof","name":"Signed local chain","nodes":[{"id":"multiply","action_ref":"calculatrice.multiply","input_bindings":{"/a":{"value":25},"/b":{"value":48}},"requires_confirmation":false},{"id":"add","action_ref":"calculatrice.add","input_bindings":{"/a":{"from_node":"multiply","source_pointer":"/result"},"/b":{"value":10}},"requires_confirmation":false}],"edges":[{"from":"multiply","to":"add"}]});let mut run=Run::new(Compiled::new(&workflow,&registry).unwrap(),Options::default()).unwrap();let mut consent=run.consent(kortex_runtime_native::permissions::now_ms(),true);let mut host=Adapter{registry,policy:Policy::default(),runtime};let status=run.advance(&mut host,&mut consent,&NativeClock,&Cancellation::default()).await;assert_eq!(status,Error::Completed);assert_eq!(run.outputs()["multiply"]["result"].as_f64(),Some(1200.0));assert_eq!(run.outputs()["add"]["result"].as_f64(),Some(1210.0));host.runtime.shutdown(id).await.unwrap();json!({"passed":true,"real_signed_wasm":true,"cloud_calls":0,"first_result":1200,"final_result":1210,"journal":run.journal()})});
    std::fs::write(
        root.join("target/workflows-runtime-proof.json"),
        serde_json::to_vec_pretty(&proof).unwrap(),
    )
    .unwrap();
}
