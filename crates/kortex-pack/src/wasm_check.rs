//! Static inspection only; never instantiates guest code.
use crate::{Issue, MAX_FILE, Result};
use std::collections::BTreeMap;
use wasmparser::{ExternalKind, Parser, Payload, TypeRef, ValType};
pub fn check(bytes: &[u8]) -> Result<()> {
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
            Payload::TableSection(r) => {
                if r.count() > 1 {
                    return Err(bad());
                }
                for table in r {
                    let t = table.map_err(|_| bad())?.ty;
                    if t.table64 || t.shared || t.maximum.is_none_or(|n| n > 65536 || t.initial > n)
                    {
                        return Err(bad());
                    }
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
    for (name, (kind, _)) in &exports {
        if !matches!(
            *name,
            "memory" | "kortex_alloc" | "kortex_dealloc" | "kortex_dispatch"
        ) && !(matches!(*name, "__heap_base" | "__data_end") && *kind == ExternalKind::Global)
        {
            return Err(bad());
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
