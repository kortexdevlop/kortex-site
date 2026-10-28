use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::LazyLock;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ToolSummary {
    pub id: String,
    pub version: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub tags: Vec<String>,
    pub icon_path: String,
    pub abi_version: String,
    pub sha256: String,
    pub published_at: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolList {
    pub items: Vec<ToolSummary>,
    pub next_cursor: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDetail {
    pub tool: ToolSummary,
    pub manifest: kortex_shared::Manifest,
    pub contract: kortex_shared::Contract,
    pub descriptor: kortex_verify::Descriptor,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolActions {
    pub tool_id: String,
    pub version: String,
    pub actions: Vec<kortex_shared::Action>,
}
fn rewrite(v: &mut Value, doc: &str) {
    match v {
        Value::Object(o) => {
            o.remove("$id");
            if let Some(Value::String(r)) = o.get_mut("$ref") {
                let (name, fragment) = r.split_once('#').unwrap_or((r.as_str(), ""));
                let name = if name.is_empty() {
                    doc
                } else {
                    match name {
                        "manifest.schema.json" => "manifest",
                        "contract.schema.json" => "contract",
                        "artifact.schema.json" => "artifact",
                        "api.schema.json" => "api",
                        _ => panic!("Unknown local schema reference"),
                    }
                };
                *r = format!("#/$defs/{name}{fragment}");
            }
            for value in o.values_mut() {
                rewrite(value, doc)
            }
        }
        Value::Array(a) => {
            for value in a {
                rewrite(value, doc)
            }
        }
        _ => {}
    }
}
fn validator(kind: &str) -> jsonschema::Validator {
    let mut defs = serde_json::Map::new();
    for (name, source) in [
        ("api", include_str!("../../shared/schemas/api.schema.json")),
        (
            "manifest",
            include_str!("../../shared/schemas/manifest.schema.json"),
        ),
        (
            "contract",
            include_str!("../../shared/schemas/contract.schema.json"),
        ),
        (
            "artifact",
            include_str!("../../shared/schemas/artifact.schema.json"),
        ),
    ] {
        let mut value = serde_json::from_str(source).unwrap();
        rewrite(&mut value, name);
        defs.insert(name.into(), value);
    }
    jsonschema::validator_for(&json!({"$schema":"https://json-schema.org/draft/2020-12/schema","$defs":defs,"$ref":format!("#/$defs/api/$defs/{kind}")})).expect("Local shared API schemas")
}
static LIST: LazyLock<jsonschema::Validator> = LazyLock::new(|| validator("ToolList"));
static DETAIL: LazyLock<jsonschema::Validator> = LazyLock::new(|| validator("ToolDetail"));
static ACTIONS: LazyLock<jsonschema::Validator> = LazyLock::new(|| validator("ToolActions"));
pub fn decode_list(v: Value) -> Result<ToolList, String> {
    if !LIST.is_valid(&v) {
        return Err("Réponse du catalogue invalide.".into());
    }
    serde_json::from_value(v).map_err(|_| "Réponse du catalogue invalide.".into())
}
pub fn decode_detail(v: Value, id: &str) -> Result<ToolDetail, String> {
    if !DETAIL.is_valid(&v) {
        return Err("Fiche invalide.".into());
    }
    let d: ToolDetail = serde_json::from_value(v).map_err(|_| "Fiche invalide.")?;
    if d.tool.id != id
        || d.manifest.id != id
        || d.contract.tool_id != id
        || d.descriptor.tool_id != id
        || d.tool.version != d.manifest.version
        || d.tool.version != d.descriptor.version
        || d.tool.sha256 != d.descriptor.sha256
        || d.tool.abi_version != d.descriptor.abi_version
        || d.tool.name != d.manifest.name
        || d.tool.summary != d.manifest.summary
        || d.tool.category != d.manifest.category
    {
        return Err("Identité de la fiche incohérente.".into());
    }
    Ok(d)
}
pub fn decode_actions(v: Value, id: &str, version: &str) -> Result<ToolActions, String> {
    if !ACTIONS.is_valid(&v) {
        return Err("Actions invalides.".into());
    }
    let d: ToolActions = serde_json::from_value(v).map_err(|_| "Actions invalides.")?;
    if d.tool_id != id || d.version != version {
        return Err("Identité des actions incohérente.".into());
    }
    Ok(d)
}
