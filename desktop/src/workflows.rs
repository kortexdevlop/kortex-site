//! Product adapter: signed installed packages, universal action dispatcher, no cloud execution.
use crate::{api::Api, config::Config, launcher};
use kortex_workflows::{Workflow, runner::Cancellation};
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
pub fn execute(
    config: &Config,
    value: Value,
    approved: bool,
    cancel: Cancellation,
    progress: Arc<Mutex<Vec<String>>>,
) -> Result<Value, String> {
    execute_with_permissions(
        config,
        value,
        approved,
        cancel,
        progress,
        crate::workflow_permissions::Permissions::default(),
    )
}
/// A hard-closed process cannot keep a runner alive. Never rewrite the marker of another live run.
pub fn recover_last_run(config: &Config) -> Result<bool, String> {
    let Ok(_lease) = crate::paths::lock(&config.data_dir, "workflow-run.lock", false) else {
        return Ok(false);
    };
    let path = config.data_dir.join("workflow.last-run.v1.json");
    if !path.exists() {
        return Ok(false);
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Record {
        run_id: String,
        status: String,
        journal: Vec<Event>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Event {
        node_id: String,
        status: String,
        attempt: u8,
        at_ms: u64,
        may_have_effects: bool,
    }
    let mut record: Record = serde_json::from_slice(&crate::paths::read(&path, 131072)?)
        .map_err(|_| "Journal invalide")?;
    if record.run_id.len() > 64 || record.journal.len() > 256 {
        return Err("Journal invalide".into());
    }
    if record.status != "running" {
        return Ok(false);
    }
    record.status = "Cancelled".into();
    crate::paths::atomic(
        &path,
        &serde_json::to_vec(&record).map_err(|_| "Journal invalide")?,
    )?;
    Ok(true)
}
pub fn execute_with_permissions(
    config: &Config,
    value: Value,
    approved: bool,
    cancel: Cancellation,
    progress: Arc<Mutex<Vec<String>>>,
    permissions: crate::workflow_permissions::Permissions,
) -> Result<Value, String> {
    let _lease = crate::paths::lock(&config.data_dir, "workflow-run.lock", false)?;
    permissions.clear();
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
    let outcome = execute_inner(
        config,
        value,
        approved,
        cancel.clone(),
        progress,
        permissions.clone(),
    );
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
    permissions.clear();
    outcome
}
fn execute_inner(
    config: &Config,
    value: Value,
    approved: bool,
    cancel: Cancellation,
    progress: Arc<Mutex<Vec<String>>>,
    permissions: crate::workflow_permissions::Permissions,
) -> Result<Value, String> {
    let end = kortex_runtime_native::permissions::now_ms() + 300000;
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
    let tool_ids = workflow
        .nodes
        .iter()
        .map(|n| n.action_ref.split('.').next().unwrap_or(""))
        .collect::<BTreeSet<_>>();
    if tool_ids.len() > 8 {
        return Err("Limite Desktop : huit outils distincts par parcours.".into());
    }
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
        packages.push(tool);
    }
    crate::workflow_host::run(config, &value, packages, cancel, progress, permissions, end)
}

#[cfg(test)]
mod input_tests {
    use super::*;
    #[test]
    fn abandoned_journal_is_cancelled_but_live_run_is_not_rewritten() {
        let t = tempfile::tempdir().unwrap();
        let c = Config {
            api_origin: "https://kortex-api-staging.userpgl.workers.dev".into(),
            channel: "staging".into(),
            data_dir: t.path().into(),
            trust_path: t.path().join("trust.json"),
            offline: true,
        };
        let path = c.data_dir.join("workflow.last-run.v1.json");
        std::fs::write(
            &path,
            br#"{"run_id":"previous","status":"running","journal":[]}"#,
        )
        .unwrap();
        let lease = crate::paths::lock(&c.data_dir, "workflow-run.lock", false).unwrap();
        assert!(!recover_last_run(&c).unwrap());
        assert!(std::fs::read_to_string(&path).unwrap().contains("running"));
        drop(lease);
        assert!(recover_last_run(&c).unwrap());
        assert!(
            std::fs::read_to_string(&path)
                .unwrap()
                .contains("Cancelled")
        );
        assert!(!recover_last_run(&c).unwrap());
    }
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
