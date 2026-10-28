#![allow(dead_code)]
// Test-only host/clock. The production crate has no injected results or alternate execution engine.
use ed25519_dalek::{Signer, SigningKey};
use kortex_capabilities::{
    Capability, Resource,
    policy::{Duration, Policy},
};
use kortex_core::{ArtifactDescriptor, VerifiedTool, dispatcher::Clock};
use kortex_workflows::{
    permissions::Scope,
    runner::{Cancellation, Host, InvokeError},
    *,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{cell::Cell, collections::BTreeMap, future::Future, rc::Rc};
pub fn block<T>(f: impl Future<Output = T>) -> T {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    for _ in 0..10000 {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
    }
    panic!("test future pending")
}
#[derive(Clone, Default)]
pub struct TestClock {
    pub now: Rc<Cell<u64>>,
    pub expire: bool,
}
impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.now.get()
    }
    async fn wait_until(&self, deadline: u64) {
        if self.expire {
            self.now.set(deadline)
        } else {
            std::future::pending::<()>().await
        }
    }
}
pub fn registry(permissions: &[&str], idempotent: bool, effect: &str) -> Registry {
    let mut m: Value =
        serde_json::from_str(include_str!("../../../../tools/calculatrice/manifest.json")).unwrap();
    let mut c: Value =
        serde_json::from_str(include_str!("../../../../tools/calculatrice/contract.json")).unwrap();
    m["permissions"] = json!(permissions);
    for a in c["actions"].as_array_mut().unwrap() {
        a["permissions"] = json!(permissions);
        a["idempotent"] = json!(idempotent);
        a["side_effect"] = json!(effect);
        a["output_schema"]["properties"]["optional"] =
            a["output_schema"]["properties"]["result"].clone();
    }
    let bytes = b"explicitly signed unit fixture, not a production module".to_vec();
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let key = SigningKey::from_bytes(&[7; 32]);
    let version = m["version"].as_str().unwrap();
    let signature = key
        .sign(format!("kortex-artifact-v1\ncalculatrice\n{version}\n1.0\n{sha256}\n").as_bytes())
        .to_bytes();
    let tool = VerifiedTool::verify(
        &m,
        &c,
        bytes,
        &ArtifactDescriptor {
            schema_version: 1,
            tool_id: "calculatrice".into(),
            version: version.into(),
            abi_version: "1.0".into(),
            sha256,
            signing_key_id: "unit-only".into(),
            signature,
        },
        &BTreeMap::from([("unit-only".into(), key.verifying_key().to_bytes())]),
    )
    .unwrap();
    let mut reg = Registry::default();
    reg.register_verified(&tool, 7).unwrap();
    reg
}
pub fn node(id: &str) -> Value {
    json!({"id":id,"action_ref":"calculatrice.add","requires_confirmation":false,"input_bindings":{"/a":{"value":25},"/b":{"value":48}}})
}
pub fn workflow() -> Value {
    json!({"workflow_version":"1.0","id":"test","name":"Unit workflow","nodes":[node("a"),node("b")],"edges":[{"from":"a","to":"b"}]})
}
pub fn chain() -> Value {
    let mut w = workflow();
    w["nodes"][1]["action_ref"] = json!("calculatrice.multiply");
    w["nodes"][1]["input_bindings"]["/a"] = json!({"from_node":"a","source_pointer":"/result"});
    w["nodes"][1]["input_bindings"]["/b"] = json!({"value":2});
    w
}
struct Dropped(Rc<Cell<bool>>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.set(true)
    }
}
pub struct TestHost {
    pub registry: Registry,
    pub policy: Policy,
    pub calls: Vec<String>,
    pub failures: usize,
    pub revoke: bool,
    pub hang: bool,
    pub cancel: Option<Cancellation>,
    pub dropped: Rc<Cell<bool>>,
    pub invalid_output: bool,
}
impl TestHost {
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            policy: Policy::default(),
            calls: vec![],
            failures: 0,
            revoke: false,
            hang: false,
            cancel: None,
            dropped: Rc::new(Cell::new(false)),
            invalid_output: false,
        }
    }
    pub fn grant(&mut self) {
        let spec = self.registry.get("calculatrice.add").unwrap();
        let caps = spec
            .action
            .permissions
            .iter()
            .map(|s| Capability::try_from(s.as_str()).unwrap())
            .collect::<Vec<_>>();
        self.policy.declare(spec.context.clone(), caps.clone());
        self.policy.enable_prompting(&spec.context);
        for cap in caps {
            self.policy
                .grant(
                    &spec.context,
                    cap,
                    &Resource::Storage {
                        key: "workflow-test".into(),
                    },
                    Duration::Session,
                    1000000,
                )
                .unwrap()
        }
    }
}
impl Host for TestHost {
    fn registry(&self) -> &Registry {
        &self.registry
    }
    fn policy(&self) -> &Policy {
        &self.policy
    }
    fn scopes(&self, _: &Node, s: &ActionSpec, _: &Value) -> Result<Vec<Scope>, Error> {
        s.action
            .permissions
            .iter()
            .map(|p| {
                Ok(Scope {
                    capability: Capability::try_from(p.as_str())
                        .map_err(|_| Error::PermissionDenied)?,
                    resource: Resource::Storage {
                        key: "workflow-test".into(),
                    },
                })
            })
            .collect()
    }
    async fn invoke(
        &mut self,
        n: &Node,
        s: &ActionSpec,
        input: Value,
        _: u64,
    ) -> Result<Value, InvokeError> {
        self.calls.push(n.id.clone());
        let _drop = Dropped(self.dropped.clone());
        if let Some(c) = &self.cancel {
            c.cancel()
        }
        if self.hang {
            std::future::pending::<()>().await
        }
        if self.revoke {
            self.policy.revoke_instance(&s.context)
        }
        if self.failures > 0 {
            self.failures -= 1;
            return Err(InvokeError { retryable: true });
        }
        if self.invalid_output {
            return Ok(json!({"result":"invalid"}));
        }
        let a = input["a"].as_f64().unwrap_or(0.0);
        let b = input["b"].as_f64().unwrap_or(0.0);
        Ok(json!({"result":if s.action.id=="multiply"{a*b}else{a+b}}))
    }
}
