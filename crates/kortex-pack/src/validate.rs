use crate::{Files, Issue, MAX_ARCHIVE, MAX_FILE, MAX_FILES, Result, archive::safe_name};
use kortex_shared::{Contract, Manifest, validate_contract, validate_manifest};
use std::collections::BTreeSet;
pub fn files(files: &Files) -> Result<(Manifest, Contract)> {
    if files.len() > MAX_FILES
        || files.values().any(|b| b.len() > MAX_FILE)
        || files.values().map(Vec::len).sum::<usize>() > MAX_ARCHIVE
    {
        return Err(Issue::new("ARCHIVE_LIMIT", ""));
    }
    let json = |name: &str| -> Result<serde_json::Value> {
        let b = files
            .get(name)
            .ok_or_else(|| Issue::new("MISSING_FILE", name))?;
        if b.len() > 1048576 {
            return Err(Issue::new("METADATA_LIMIT", name));
        }
        serde_json::from_slice(b).map_err(|_| Issue::new("INVALID_JSON", name))
    };
    let m = validate_manifest(&json("manifest.json")?)
        .map_err(|_| Issue::new("INVALID_MANIFEST", "manifest.json"))?;
    let c = validate_contract(&json("contract.json")?)
        .map_err(|_| Issue::new("INVALID_CONTRACT", "contract.json"))?;
    if m.id != c.tool_id
        || c.actions
            .iter()
            .any(|a| a.permissions.iter().any(|p| !m.permissions.contains(p)))
    {
        return Err(Issue::new("METADATA_MISMATCH", "contract.json"));
    }
    if !m.icon_path.starts_with("assets/") || !safe_name(&m.icon_path) {
        return Err(Issue::new("INVALID_ASSET", &m.icon_path));
    }
    let allowed = BTreeSet::from([
        "manifest.json",
        "contract.json",
        "module.wasm",
        m.icon_path.as_str(),
    ]);
    for name in files.keys() {
        if !allowed.contains(name.as_str()) {
            return Err(Issue::new("UNREFERENCED_FILE", name));
        }
    }
    for name in &allowed {
        if !files.contains_key(*name) {
            return Err(Issue::new("MISSING_FILE", name));
        }
    }
    if files[&m.icon_path].is_empty() {
        return Err(Issue::new("INVALID_ASSET", &m.icon_path));
    }
    wasm(&files["module.wasm"])?;
    Ok((m, c))
}
/// Backwards-compatible entry point to the shared static validator.
pub fn wasm(bytes: &[u8]) -> Result<()> {
    crate::wasm_check::check(bytes)
}
