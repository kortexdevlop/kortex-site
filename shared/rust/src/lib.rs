use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, sync::LazyLock};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Violation {
    pub code: String,
    pub path: String,
    pub message: String,
}
fn violation(code: &str, message: &str) -> Vec<Violation> {
    vec![Violation {
        code: code.into(),
        path: String::new(),
        message: message.into(),
    }]
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format_version: String,
    pub id: String,
    pub version: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub seo: Seo,
    pub icon_path: String,
    pub abi_version: String,
    pub ui_schema_version: String,
    pub permissions: Vec<String>,
    pub entrypoint: String,
    pub min_runtime: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seo {
    pub title: String,
    pub description: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub contract_version: String,
    pub tool_id: String,
    pub actions: Vec<Action>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub permissions: Vec<String>,
    pub side_effect: String,
    pub idempotent: bool,
    pub timeout_ms: u32,
}
static MANIFEST: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    jsonschema::validator_for(
        &serde_json::from_str(include_str!("../../schemas/manifest.schema.json"))
            .expect("schema JSON"),
    )
    .expect("manifest schema")
});
static CONTRACT: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    jsonschema::validator_for(
        &serde_json::from_str(include_str!("../../schemas/contract.schema.json"))
            .expect("schema JSON"),
    )
    .expect("contract schema")
});
fn shape(validator: &jsonschema::Validator, value: &Value) -> Result<(), Vec<Violation>> {
    if serde_json::to_vec(value).map_or(true, |v| v.len() > 1048576) {
        return Err(violation("LIMIT", "JSON document exceeds 1 MiB"));
    }
    if validator.is_valid(value) {
        Ok(())
    } else {
        Err(violation("SCHEMA", "Schema violation"))
    }
}
fn bounded(v: &Value, depth: usize) -> bool {
    if depth > 16 {
        return false;
    }
    for (min, max) in [
        ("minimum", "maximum"),
        ("minLength", "maxLength"),
        ("minItems", "maxItems"),
    ] {
        if let (Some(a), Some(b)) = (v[min].as_f64(), v[max].as_f64()) {
            if a > b {
                return false;
            }
        }
    }
    if let Some(required) = v["required"].as_array() {
        if required
            .iter()
            .any(|key| v["properties"].get(key.as_str().unwrap_or("")).is_none())
        {
            return false;
        }
    }
    if let Some(values) = v["enum"].as_array() {
        if values.iter().any(|x| {
            x.as_f64()
                .is_some_and(|n| !n.is_finite() || n.abs() > 9007199254740991.0)
        }) {
            return false;
        }
    }
    if let Some(properties) = v["properties"].as_object() {
        if properties.values().any(|x| !bounded(x, depth + 1)) {
            return false;
        }
    }
    v.get("items").is_none_or(|x| bounded(x, depth + 1))
}
pub fn validate_manifest(value: &Value) -> Result<Manifest, Vec<Violation>> {
    shape(&MANIFEST, value)?;
    serde_json::from_value(value.clone()).map_err(|_| violation("SCHEMA", "Schema violation"))
}
pub fn validate_contract(value: &Value) -> Result<Contract, Vec<Violation>> {
    shape(&CONTRACT, value)?;
    let contract: Contract = serde_json::from_value(value.clone())
        .map_err(|_| violation("SCHEMA", "Schema violation"))?;
    let mut ids = HashSet::new();
    if contract.actions.iter().any(|a| {
        !ids.insert(&a.id) || !bounded(&a.input_schema, 0) || !bounded(&a.output_schema, 0)
    }) {
        return Err(violation("INVARIANT", "Semantic invariant violated"));
    }
    Ok(contract)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_fixtures() {
        for (file, valid, kind) in [
            ("manifest.valid.json", true, "manifest"),
            ("invalid-manifest.json", false, "manifest"),
            ("contract.valid.json", true, "contract"),
            ("invalid-contract.json", false, "contract"),
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../fixtures")
                .join(file);
            let value: Value =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let accepted = if kind == "manifest" {
                validate_manifest(&value).is_ok()
            } else {
                validate_contract(&value).is_ok()
            };
            assert_eq!(accepted, valid, "{file}");
        }
    }
    #[test]
    fn cross_language_matrix() {
        let Ok(path) = std::env::var("KORTEX_CONTRACT_CASES") else {
            return;
        };
        let cases: Vec<Value> =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let results: Vec<bool> = cases
            .iter()
            .map(|c| {
                if c["kind"] == "manifest" {
                    validate_manifest(&c["value"]).is_ok()
                } else {
                    validate_contract(&c["value"]).is_ok()
                }
            })
            .collect();
        println!("KORTEX_MATRIX:{}", serde_json::to_string(&results).unwrap());
    }
}
