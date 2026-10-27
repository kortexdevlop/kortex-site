//! Canonical .tool construction and bounded, memory-only validation.
pub mod archive;
pub mod validate;
pub mod wasm_check;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub type Files = BTreeMap<String, Vec<u8>>;
pub const MAX_ARCHIVE: usize = 25 * 1024 * 1024;
pub const MAX_FILE: usize = 20 * 1024 * 1024;
pub const MAX_FILES: usize = 256;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Issue {
    pub code: String,
    pub path: String,
}
impl Issue {
    pub fn new(code: &str, path: &str) -> Self {
        Self {
            code: code.into(),
            path: path.into(),
        }
    }
}
pub type Result<T> = std::result::Result<T, Issue>;
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub issues: Vec<Issue>,
    pub files: usize,
    pub unpacked_bytes: usize,
    pub tool_id: Option<String>,
    pub version: Option<String>,
}
pub fn inspect(bytes: &[u8]) -> ValidationReport {
    match archive::read(bytes).and_then(|files| {
        let metadata = validate::files(&files)?;
        Ok((files, metadata))
    }) {
        Ok((files, (m, _))) => ValidationReport {
            valid: true,
            issues: vec![],
            files: files.len(),
            unpacked_bytes: files.values().map(Vec::len).sum(),
            tool_id: Some(m.id),
            version: Some(m.version),
        },
        Err(issue) => ValidationReport {
            valid: false,
            issues: vec![issue],
            files: 0,
            unpacked_bytes: 0,
            tool_id: None,
            version: None,
        },
    }
}
/// Deterministic stored ZIP, sorted names, DOS epoch 1980-01-01, Unix 0644.
/// JSON metadata is serialized with sorted keys and no insignificant whitespace.
pub fn package(mut files: Files) -> Result<Vec<u8>> {
    validate::files(&files)?;
    for name in ["manifest.json", "contract.json"] {
        let v: serde_json::Value =
            serde_json::from_slice(&files[name]).map_err(|_| Issue::new("INVALID_JSON", name))?;
        files.insert(
            name.into(),
            serde_json::to_vec(&canonical(v)).map_err(|_| Issue::new("INVALID_JSON", name))?,
        );
    }
    let bytes = archive::write(&files)?;
    let report = inspect(&bytes);
    if !report.valid {
        return Err(report.issues[0].clone());
    }
    Ok(bytes)
}
fn canonical(value: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .map(|(k, v)| (k, canonical(v)))
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.into_iter().map(canonical).collect()),
        other => other,
    }
}
