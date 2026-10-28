//! Universal signed-action adapter. Capability operations always use NativeServices/Gateway.
use crate::{
    config::Config,
    workflow_permissions::{self, Permissions},
};
use kortex_capabilities::{
    Capability, Resource,
    policy::{Decision, Duration, Policy},
};
use kortex_core::{Runtime, VerifiedTool, dispatcher::Clock};
use kortex_runtime_native::{
    HostedNativeEngine,
    network::NativeNetwork,
    permissions::{CapabilityInbox, NativeHost, NativeServices, now_ms},
    storage::NativeStorage,
};
use kortex_workflows::{
    permissions::Scope,
    runner::{Cancellation, Host, InvokeError, Options, Run},
    *,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
struct NativeClock;
impl Clock for NativeClock {
    fn now_ms(&self) -> u64 {
        now_ms()
    }
    async fn wait_until(&self, end: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(
            end.saturating_sub(now_ms()),
        ))
        .await
    }
}
type Engine = Runtime<HostedNativeEngine, NativeClock>;
struct Loaded {
    runtime: Engine,
    inbox: CapabilityInbox,
    id: u64,
}
#[derive(Clone)]
struct Prepared {
    node: String,
    input: Value,
    scopes: Vec<Scope>,
    approved_at: BTreeMap<String, u64>,
}
struct LocalHost {
    registry: Registry,
    tools: BTreeMap<String, Loaded>,
    services: NativeServices,
    network: NativeNetwork,
    progress: Arc<Mutex<Vec<String>>>,
    permissions: Permissions,
    cancel: Cancellation,
    end: u64,
    prepared: Mutex<Option<Prepared>>,
}
impl LocalHost {
    fn prepare(&self, node: &Node, spec: &ActionSpec, input: &Value) -> Result<Prepared, Error> {
        if let Some(p) = self
            .prepared
            .lock()
            .map_err(|_| Error::PermissionDenied)?
            .as_ref()
        {
            if p.node == node.id && p.input == *input {
                return Ok(p.clone());
            }
        }
        let mut scopes = Vec::new();
        let mut approved_at = BTreeMap::new();
        let mut names = spec.action.permissions.clone();
        names.sort_by_key(|n| {
            Capability::try_from(n.as_str())
                .map(|c| c.needs_gesture())
                .unwrap_or(false)
        });
        for name in names {
            let cap = Capability::try_from(name.as_str()).map_err(|_| Error::PermissionDenied)?;
            let (value, at) = self.permissions.ask(
                &node.id,
                &spec.context.tool_id,
                &spec.context.version,
                &name,
                workflow_permissions::suggested(cap, input),
                workflow_permissions::editable(cap),
                &self.cancel,
                self.end,
            )?;
            let resource = workflow_permissions::resource(cap, value)?;
            if let Resource::Network { url } = &resource {
                let origin = NativeNetwork::url(url)
                    .map_err(|_| Error::PermissionDenied)?
                    .origin()
                    .ascii_serialization();
                self.network
                    .allow_origin(origin)
                    .map_err(|_| Error::PermissionDenied)?;
            }
            approved_at.insert(name, at);
            scopes.push(Scope {
                capability: cap,
                resource,
            });
        }
        let p = Prepared {
            node: node.id.clone(),
            input: input.clone(),
            scopes,
            approved_at,
        };
        *self.prepared.lock().map_err(|_| Error::PermissionDenied)? = Some(p.clone());
        Ok(p)
    }
}
impl Host for LocalHost {
    fn registry(&self) -> &Registry {
        &self.registry
    }
    fn policy(&self) -> &Policy {
        &self.services.gateway.policy
    }
    fn scopes(&self, n: &Node, s: &ActionSpec, input: &Value) -> Result<Vec<Scope>, Error> {
        Ok(self.prepare(n, s, input)?.scopes)
    }
    async fn invoke(
        &mut self,
        n: &Node,
        s: &ActionSpec,
        input: Value,
        deadline: u64,
    ) -> Result<Value, InvokeError> {
        let fail = || InvokeError { retryable: false };
        let prepared = self.prepare(n, s, &input).map_err(|_| fail())?;
        if let Ok(mut p) = self.progress.lock() {
            p.push(format!("{} : en cours", n.id))
        }
        let tool = self.tools.get_mut(&s.context.tool_id).ok_or_else(fail)?;
        let mut reply = tool
            .runtime
            .invoke_action(tool.id, &s.action.id, input)
            .await
            .map_err(|_| fail())?;
        if !reply.ok {
            return Err(fail());
        }
        let mut count = 0;
        while let Some(request) = tool.inbox.pop() {
            count += 1;
            if count > 128 || self.cancel.is_cancelled() || now_ms() >= deadline {
                return Err(fail());
            }
            let cap = Capability::try_from(request.capability.as_str()).map_err(|_| fail())?;
            let resource =
                kortex_runtime_native::permissions::resource(&request).map_err(|_| fail())?;
            let exact = prepared.scopes.iter().any(|v| {
                v.capability == cap && v.resource.scope(cap).ok() == resource.scope(cap).ok()
            });
            if !exact || !s.action.permissions.contains(&request.capability) {
                return Err(fail());
            }
            let outcome = if cap.needs_gesture() {
                let at = prepared
                    .approved_at
                    .get(&request.capability)
                    .ok_or_else(fail)?;
                if now_ms() < *at || now_ms().saturating_sub(*at) >= 1000 {
                    self.permissions
                        .ask(
                            &n.id,
                            &s.context.tool_id,
                            &s.context.version,
                            &request.capability,
                            workflow_permissions::suggested(
                                cap,
                                &Value::Object(request.payload.clone()),
                            ),
                            false,
                            &self.cancel,
                            deadline.min(self.end),
                        )
                        .map_err(|_| fail())?;
                }
                self.services
                    .execute_existing_with_gesture(&s.context, &request)
                    .await
            } else {
                self.services.execute_existing(&s.context, &request).await
            };
            let ok = outcome.is_ok();
            tool.inbox.complete(&request.request_id);
            reply = tool
                .runtime
                .dispatch_capability_result(tool.id, request.request_id, outcome)
                .await
                .map_err(|_| fail())?;
            if !ok || !reply.ok {
                return Err(fail());
            }
        }
        *self.prepared.lock().map_err(|_| fail())? = None;
        if let Ok(mut p) = self.progress.lock() {
            p.push(format!("{} : terminé", n.id))
        }
        reply.data.map(Value::Object).ok_or_else(fail)
    }
}
pub(crate) fn run(
    config: &Config,
    value: &Value,
    packages: Vec<VerifiedTool>,
    cancel: Cancellation,
    progress: Arc<Mutex<Vec<String>>>,
    permissions: Permissions,
    end: u64,
) -> Result<Value, String> {
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|_| "Moteur indisponible")?;
    executor.block_on(async{
  let network=NativeNetwork::default();let storage=NativeStorage::new(config.data_dir.join("data")).map_err(|_|"Stockage indisponible")?;let mut services=NativeServices::new(NativeHost::new(storage,network.clone()));let mut registry=Registry::default();let mut tools=BTreeMap::new();
  for package in packages{if cancel.is_cancelled(){return Err("Parcours arrêté.".into())}let name=package.manifest().id.clone();let inbox=CapabilityInbox::default();let mut runtime=Runtime::new(HostedNativeEngine::new(inbox.clone()),NativeClock);let id=runtime.load(package.clone()).await.map_err(|_|"Paquet refusé")?;registry.register_verified(&package,id).map_err(|_|"Actions refusées")?;let context=runtime.prepare_capabilities(id,&mut services.gateway).map_err(|_|"Permissions refusées")?;services.gateway.policy.enable_prompting(&context);tools.insert(name,Loaded{runtime,inbox,id});}
  let compiled=Compiled::new(value,&registry).map_err(|_|"Paramètres ou liaisons incompatibles")?;
  let mut host=LocalHost{registry,tools,services,network,progress,permissions,cancel:cancel.clone(),end,prepared:Mutex::new(None)};
  let result=async{
   for (name,loaded) in &mut host.tools{loaded.runtime.initialize(loaded.id).await.map_err(|_|"Initialisation refusée")?;let owner=loaded.runtime.capability_context(loaded.id).map_err(|_|"Permissions refusées")?;let mut count=0;
    while let Some(request)=loaded.inbox.pop(){count+=1;if count>128||cancel.is_cancelled()||now_ms()>=end{return Err("Initialisation interrompue".into())}let cap=Capability::try_from(request.capability.as_str()).map_err(|_|"Capacité inconnue")?;let resource=kortex_runtime_native::permissions::resource(&request).map_err(|_|"Ressource invalide")?;let display=workflow_permissions::suggested(cap,&Value::Object(request.payload.clone()));host.permissions.ask("initialisation",name,&owner.version,&request.capability,display,false,&cancel,end).map_err(|_|"Accès initial refusé")?;
     if let Resource::Network{url}=&resource{host.network.allow_origin(NativeNetwork::url(url).map_err(|_|"Adresse refusée")?.origin().ascii_serialization()).map_err(|_|"Adresse refusée")?;}
     let outcome=host.services.fulfill(&owner,&request,true,Duration::Once).await;let ok=outcome.is_ok();loaded.inbox.complete(&request.request_id);loaded.runtime.dispatch_capability_result(loaded.id,request.request_id,outcome).await.map_err(|_|"Initialisation refusée")?;if !ok{return Err("Accès initial refusé".into())}
    }
   }
   let mut run=Run::new(compiled,Options::default()).map_err(|_|"Parcours refusé")?;let mut consent=run.consent(now_ms(),true);let nodes=run.plan().workflow().nodes.iter().map(|n|n.id.clone()).collect::<Vec<_>>();consent.approve_batch(run.plan(),&nodes).map_err(|_|"Confirmation refusée")?;
   let status=loop{let status=run.advance(&mut host,&mut consent,&NativeClock,&cancel).await;if cancel.is_cancelled(){break Error::Cancelled}if status!=Error::AwaitingConsent{break status}if cancel.is_cancelled()||now_ms()>=end{cancel.cancel();continue}
    let node=run.pending_node().ok_or("Étape introuvable")?.clone();let spec=host.registry.get(&node.action_ref).ok_or("Action introuvable")?.clone();let input=run.pending_input().map_err(|_|"Paramètres invalides")?;let prepared=host.prepare(&node,&spec,&input).map_err(|_|"Autorisation refusée")?;
    for scope in prepared.scopes{match host.services.gateway.policy.consent_decision(&spec.context,scope.capability,&scope.resource,now_ms()){Decision::Ask=>host.services.gateway.policy.grant(&spec.context,scope.capability,&scope.resource,Duration::Once,end).map_err(|_|"Autorisation refusée")?,Decision::Deny=>return Err("Accès révoqué".into()),_=>{}}}
   };
   Ok(json!({"status":format!("{status:?}"),"outputs":run.outputs(),"journal":run.journal(),"executed_locally":true,"shareable":false}))
  }.await;
  for loaded in host.tools.values_mut(){if let Ok(owner)=loaded.runtime.capability_context(loaded.id){host.services.revoke(&owner)}let _=loaded.runtime.shutdown(loaded.id).await;}
  result
 })
}
