use crate::config::Config;
use serde_json::Value;

fn safe(value: &Value, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    match value {
        Value::Array(a) => a.iter().all(|v| safe(v, depth + 1)),
        Value::Object(o) => o.iter().all(|(k, v)| {
            (k == "data_base64" && v == ""
                || !matches!(
                    k.to_lowercase().as_str(),
                    "data_base64"
                        | "content_base64"
                        | "data_text"
                        | "bytes"
                        | "file_path"
                        | "path"
                        | "handle"
                        | "file_handle"
                        | "content"
                ))
                && safe(v, depth + 1)
        }),
        Value::String(s) => {
            !s.starts_with("/home/")
                && !s.starts_with("/Users/")
                && !(s.as_bytes().get(1) == Some(&b':') && s.as_bytes().get(2) == Some(&b'\\'))
        }
        _ => true,
    }
}
pub fn definition(value: &Value) -> Result<String, String> {
    if value["nodes"].as_array().is_some_and(|v|v.len()>50)||value["edges"].as_array().is_some_and(|v|v.len()>100){return Err("Limite de parcours dépassée.".into())}
    let text = serde_json::to_string(value).map_err(|_| "Parcours invalide")?;
    if text.len() > 262144 {
        return Err("Limite de sauvegarde : 256 Ko.".into());
    }
    let schema: Value =
        serde_json::from_str(include_str!("../../shared/schemas/workflow.schema.json"))
            .map_err(|_| "Schéma indisponible")?;
    if !jsonschema::validator_for(&schema)
        .map_err(|_| "Schéma indisponible")?
        .is_valid(value)
    {
        return Err("Parcours invalide.".into());
    }
    for node in value["nodes"].as_array().ok_or("Parcours invalide")? {
        for (path, binding) in node["input_bindings"]
            .as_object()
            .ok_or("Paramètres invalides")?
        {
            if let Some(v) = binding.get("value") {
                if !safe(v, 0)
                    || path.split('/').any(|k| {
                        !(k == "data_base64" && v == "") && !safe(&serde_json::json!({k:null}), 0)
                    })
                {
                    return Err(
                        "Retirez les contenus de fichiers et les chemins avant de sauvegarder."
                            .into(),
                    );
                }
            }
        }
    }
    Ok(text)
}
pub fn save(c: &Config, value: &Value) -> Result<(), String> {
    let text = definition(value)?;
    crate::paths::atomic(&c.data_dir.join("workflow.v1.json"), text.as_bytes())
}
pub fn load(c: &Config) -> Result<Value, String> {
    let bytes = crate::paths::read(&c.data_dir.join("workflow.v1.json"), 262144)?;
    let value = serde_json::from_slice(&bytes).map_err(|_| "Sauvegarde invalide")?;
    definition(&value)?;
    Ok(value)
}
pub fn sync(c: &Config, value: &Value, revision: u64, owner: &str) -> Result<Value, String> {
    definition(value)?;
    crate::account::workflow_request(
        c,
        reqwest::Method::PUT,
        Some(value["id"].as_str().ok_or("Identifiant invalide")?),
        Some(value.clone()),
        Some(revision),
        Some(owner),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn value() -> Value {
        json!({"workflow_version":"1.0","id":"local","name":"Local","nodes":[{"id":"one","action_ref":"calculatrice.add","input_bindings":{"/a":{"value":1},"/b":{"value":2}},"requires_confirmation":false}],"edges":[]})
    }
    #[test]
    fn local_definition_roundtrips_without_account() {
        let temp = tempfile::tempdir().unwrap();
        let c = Config {
            api_origin: "https://kortex-api-staging.userpgl.workers.dev".into(),
            channel: "staging".into(),
            data_dir: temp.path().into(),
            trust_path: temp.path().join("trust.json"),
            offline: true,
        };
        save(&c, &value()).unwrap();
        assert_eq!(load(&c).unwrap(), value());
    }
    #[test]
    fn rejects_private_payloads_handles_paths_and_foreign_state() {
        for key in ["data_base64", "file_handle", "path", "bytes"] {
            let mut w = value();
            w["nodes"][0]["input_bindings"] = json!({"":{"value":{key:"private"}}});
            assert!(definition(&w).is_err())
        }
        let mut w = value();
        w["results"] = json!({"private":"never-save"});
        assert!(definition(&w).is_err());
        let mut w = value();
        w["nodes"][0]["input_bindings"]["/a"] = json!({"value":"C:\\Users\\secret.txt"});
        assert!(definition(&w).is_err());
    }
}
