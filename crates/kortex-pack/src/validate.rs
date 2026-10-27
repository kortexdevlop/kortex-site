use crate::{Files, Issue, MAX_ARCHIVE, MAX_FILE, MAX_FILES, Result, archive::safe_name};
use kortex_shared::{Contract, Manifest, validate_contract, validate_manifest};
use std::collections::{BTreeMap, BTreeSet};
use wasmparser::{ExternalKind, Parser, Payload, TypeRef, ValType};
pub fn files(files: &Files) -> Result<(Manifest, Contract)> {
    if files.len() > MAX_FILES || files.values().map(Vec::len).sum::<usize>() > MAX_ARCHIVE {
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
pub fn wasm(bytes: &[u8]) -> Result<()> {
    let bad = || Issue::new("INVALID_WASM_ABI", "module.wasm");
    if bytes.len() > MAX_FILE || bytes.get(..8) != Some(b"\0asm\x01\0\0\0") {
        return Err(bad());
    }
    wasmparser::Validator::new()
        .validate_all(bytes)
        .map_err(|_| bad())?;
    let mut types = Vec::new();
    let mut functions = Vec::new();
    let mut memories = Vec::new();
    let mut exports = BTreeMap::new();
    let mut imported = false;
    for part in Parser::new(0).parse_all(bytes) {
        match part.map_err(|_| bad())? {
            Payload::TypeSection(r) => {
                for t in r.into_iter_err_on_gc_types() {
                    types.push(t.map_err(|_| bad())?);
                }
            }
            Payload::ImportSection(r) => {
                for i in r.into_imports() {
                    let i = i.map_err(|_| bad())?;
                    if i.module != "kortex_v1" || i.name != "host_enqueue" || imported {
                        return Err(bad());
                    }
                    let TypeRef::Func(index) = i.ty else {
                        return Err(bad());
                    };
                    let ty = types.get(index as usize).ok_or_else(bad)?;
                    if ty.params() != [ValType::I32, ValType::I32] || ty.results() != [ValType::I32]
                    {
                        return Err(bad());
                    }
                    functions.push(index);
                    imported = true;
                }
            }
            Payload::FunctionSection(r) => {
                for f in r {
                    functions.push(f.map_err(|_| bad())?);
                }
            }
            Payload::MemorySection(r) => {
                for m in r {
                    memories.push(m.map_err(|_| bad())?);
                }
            }
            Payload::ExportSection(r) => {
                for e in r {
                    let e = e.map_err(|_| bad())?;
                    exports.insert(e.name, (e.kind, e.index));
                }
            }
            Payload::StartSection { .. } => return Err(bad()),
            _ => {}
        }
    }
    if memories.len() != 1 {
        return Err(bad());
    }
    let m = memories[0];
    if m.memory64
        || m.shared
        || m.page_size_log2.is_some_and(|n| n != 16)
        || m.maximum.is_none_or(|n| n > 1024 || m.initial > n)
        || exports.get("memory") != Some(&(ExternalKind::Memory, 0))
    {
        return Err(bad());
    }
    for (name, params, results) in [
        ("kortex_alloc", vec![ValType::I32], vec![ValType::I32]),
        ("kortex_dealloc", vec![ValType::I32, ValType::I32], vec![]),
        (
            "kortex_dispatch",
            vec![ValType::I32, ValType::I32],
            vec![ValType::I64],
        ),
    ] {
        let (kind, index) = exports.get(name).ok_or_else(bad)?;
        if *kind != ExternalKind::Func {
            return Err(bad());
        }
        let ty = types
            .get(*functions.get(*index as usize).ok_or_else(bad)? as usize)
            .ok_or_else(bad)?;
        if ty.params() != params || ty.results() != results {
            return Err(bad());
        }
    }
    Ok(())
}
