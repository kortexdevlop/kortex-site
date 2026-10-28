//! Product adapter: signed installed packages, universal action dispatcher, no cloud execution.
use crate::{api::Api, config::Config, launcher};
use kortex_capabilities::policy::Policy;
use kortex_core::{Runtime, dispatcher::Clock};
use kortex_workflows::{
    permissions::Scope,
    runner::{Cancellation, Host, InvokeError, Options, Run},
    *,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};
/// Trusted UI file selection only; the path and file bytes never modify the saved definition.
pub fn bind_file(
    value: &Value,
    node_id: &str,
    path: &std::path::Path,
    approved: bool,
    cancel: &Cancellation,
) -> Result<Value, String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    if !approved {
        return Err("Autorisez la lecture locale du fichier avant de lancer le parcours.".into());
    }
    if cancel.is_cancelled() {
        return Err("Parcours arrêté.".into());
    }
    let mut local = value.clone();
    let node = local["nodes"]
        .as_array_mut()
        .and_then(|nodes| nodes.iter_mut().find(|n| n["id"] == node_id))
        .ok_or("Étape de fichier introuvable")?;
    let owner = kortex_capabilities::policy::Context {
        tool_id: "kortex-workflow-input".into(),
        version: "1.0.0".into(),
        instance_id: 1,
    };
    let files = kortex_runtime_native::files::NativeFiles::default();
    let selected = files
        .select(
            &owner,
            Some(path),
            kortex_runtime_native::permissions::now_ms(),
        )
        .map_err(|_| "Fichier local refusé")?;
    let size = selected["size"].as_u64().ok_or("Fichier invalide")?;
    if size > 786000 {
        return Err("Ce fichier dépasse la limite locale de 786 Ko.".into());
    }
    let backend = selected["backend"].as_str().ok_or("Fichier invalide")?;
    let mut bytes = Vec::new();
    while bytes.len() < (size as usize) {
        if cancel.is_cancelled() {
            return Err("Parcours arrêté.".into());
        }
        let part = files
            .read(
                &owner,
                backend,
                bytes.len() as u64,
                ((size as usize) - bytes.len()).min(262144),
                kortex_runtime_native::permissions::now_ms(),
            )
            .map_err(|_| "Lecture interrompue")?;
        let chunk = STANDARD
            .decode(part["data_base64"].as_str().ok_or("Lecture invalide")?)
            .map_err(|_| "Lecture invalide")?;
        if chunk.is_empty() {
            return Err("Le fichier a changé.".into());
        }
        bytes.extend(chunk);
    }
    files.revoke(&owner);
    node["input_bindings"]["/data_base64"] = json!({"value":STANDARD.encode(bytes)});
    node["requires_confirmation"] = json!(true);
    Ok(local)
}
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
struct LocalHost {
    registry: Registry,
    policy: Policy,
    runtime: Runtime<kortex_runtime_native::NativeGuestEngine, NativeClock>,
    progress: Arc<Mutex<Vec<String>>>,
}
impl Host for LocalHost {
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
        node: &Node,
        s: &ActionSpec,
        input: Value,
        _: u64,
    ) -> Result<Value, InvokeError> {
        if let Ok(mut p) = self.progress.lock() {
            p.push(format!("{} : en cours", node.id));
        }
        let reply = self
            .runtime
            .invoke_action(s.context.instance_id, &s.action.id, input)
            .await
            .map_err(|_| InvokeError { retryable: false })?;
        if !reply.ok {
            return Err(InvokeError { retryable: false });
        }
        let data = reply
            .data
            .map(Value::Object)
            .ok_or(InvokeError { retryable: false })?;
        if let Ok(mut p) = self.progress.lock() {
            p.push(format!("{} : terminé", node.id));
        }
        Ok(data)
    }
}
pub fn execute(
    config: &Config,
    value: Value,
    approved: bool,
    cancel: Cancellation,
    progress: Arc<Mutex<Vec<String>>>,
) -> Result<Value, String> {
    if !approved {
        return Err("Vérifiez et approuvez le parcours avant de le lancer.".into());
    }
    let run_id = crate::paths::nonce()?;
    let path = config.data_dir.join("workflow.last-run.v1.json");
    let _ = crate::paths::atomic(
        &path,
        serde_json::to_vec(&json!({"run_id":run_id,"status":"running","journal":[]}))
            .unwrap()
            .as_slice(),
    );
    let outcome = execute_inner(config, value, approved, cancel.clone(), progress);
    let (status, journal) = match &outcome {
        Ok(v) => (
            v["status"].as_str().unwrap_or("failed"),
            v["journal"].clone(),
        ),
        Err(_) => (
            if cancel.is_cancelled() {
                "Cancelled"
            } else {
                "Failed"
            },
            json!([]),
        ),
    };
    let _ = crate::paths::atomic(
        &path,
        serde_json::to_vec(&json!({"run_id":run_id,"status":status,"journal":journal}))
            .unwrap()
            .as_slice(),
    );
    outcome
}
fn execute_inner(
    config: &Config,
    value: Value,
    approved: bool,
    cancel: Cancellation,
    progress: Arc<Mutex<Vec<String>>>,
) -> Result<Value, String> {
    if serde_json::to_vec(&value)
        .map_err(|_| "Parcours invalide")?
        .len()
        > 1048576
    {
        return Err("Parcours trop volumineux".into());
    }
    let workflow =
        serde_json::from_value::<Workflow>(value.clone()).map_err(|_| "Parcours invalide.")?;
    if workflow.nodes.len() > 50 || workflow.edges.len() > 100 {
        return Err("Limite de parcours dépassée.".into());
    }
    if workflow.nodes.is_empty() {
        return Err("Ajoutez une étape.".into());
    }
    if !approved {
        return Err("Vérifiez et approuvez le parcours avant de le lancer.".into());
    }
    let api = Api::new(config).map_err(|e| e.to_string())?;
    let mut packages = Vec::new();
    let mut leases = Vec::new();
    let tool_ids=workflow.nodes.iter().map(|n|n.action_ref.split('.').next().unwrap_or("")).collect::<BTreeSet<_>>();
    if tool_ids.len()>8{return Err("Limite Desktop : huit outils distincts par parcours.".into())}
    for id in tool_ids {
        if cancel.is_cancelled() {
            return Err("Parcours arrêté.".into());
        }
        let launch = launcher::prepare(config, &api, id)?;
        let installed = launch.installed;
        leases.push(launch.lease);
        if !config.offline {
            let published = api.detail_fresh(id).map_err(|e| e.to_string())?;
            if published.tool.version != installed.version
                || published.descriptor.sha256 != installed.sha256
            {
                return Err(
                    "Une version a changé : actualisez les outils avant de lancer le parcours."
                        .into(),
                );
            }
        }
        let tool = kortex_runtime_desktop::app::open_verified_with_pin(
            installed.archive.to_str().ok_or("Paquet invalide")?,
            installed.trust.to_str().ok_or("Clés invalides")?,
            Some(&installed.trust_seal),
        )?;
        if tool.contract().actions.iter().any(|a| {
            workflow
                .nodes
                .iter()
                .any(|n| n.action_ref == format!("{}.{}", id, a.id))
                && !a.permissions.is_empty()
        }) {
            return Err("Ce parcours demande un accès sensible. Cet accès n’est pas encore disponible dans le lanceur de parcours Desktop ; ouvrez l’outil directement.".into());
        }
        packages.push(tool);
    }
    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|_| "Moteur indisponible")?;
    executor.block_on(async{let mut runtime=Runtime::new(kortex_runtime_native::NativeGuestEngine,NativeClock);let mut registry=Registry::default();let mut ids=Vec::new();
  let outcome=async{for tool in packages{let id=runtime.load(tool.clone()).await.map_err(|_|"Paquet refusé")?;ids.push(id);registry.register_verified(&tool,id).map_err(|_|"Actions refusées")?;}
   let compiled=Compiled::new(&value,&registry).map_err(|_|"Paramètres ou liaisons incompatibles")?;for id in &ids{runtime.initialize(*id).await.map_err(|_|"Initialisation refusée")?;}let mut run=Run::new(compiled,Options::default()).map_err(|_|"Parcours refusé")?;let mut consent=run.consent(kortex_runtime_native::permissions::now_ms(),true);let nodes=workflow.nodes.iter().map(|n|n.id.clone()).collect::<Vec<_>>();consent.approve_batch(run.plan(),&nodes).map_err(|_|"Confirmation refusée")?;
   let mut host=LocalHost{registry,policy:Policy::default(),runtime,progress};let status=run.advance(&mut host,&mut consent,&NativeClock,&cancel).await;for id in &ids{let _=host.runtime.shutdown(*id).await;}Ok::<_,String>(json!({"status":format!("{status:?}"),"outputs":run.outputs(),"journal":run.journal(),"executed_locally":true,"shareable":false}))
  }.await;outcome
 })
}

#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn file_requires_consent_and_remains_ephemeral() {
        let t = tempfile::tempdir().unwrap();
        let file = t.path().join("private.txt");
        std::fs::write(&file, b"abc").unwrap();
        let w = json!({"nodes":[{"id":"hash","input_bindings":{"/data_base64":{"value":""}},"requires_confirmation":false}]});
        assert!(
            bind_file(&w, "hash", &file, false, &Cancellation::default())
                .unwrap_err()
                .contains("Autorisez")
        );
        let bound = bind_file(&w, "hash", &file, true, &Cancellation::default()).unwrap();
        assert_eq!(
            bound["nodes"][0]["input_bindings"]["/data_base64"]["value"],
            "YWJj"
        );
        assert_eq!(w["nodes"][0]["input_bindings"]["/data_base64"]["value"], "");
        let cancelled = Cancellation::default();
        cancelled.cancel();
        assert!(bind_file(&w, "hash", &file, true, &cancelled).is_err());
    }
}
