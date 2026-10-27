use crate::{
    Result,
    discovery::{Tool, link},
    read_bounded,
};
use std::path::{Path, PathBuf};
pub fn collect(tool: &Tool, module: &Path) -> Result<kortex_pack::Files> {
    let mut files = kortex_pack::Files::new();
    for name in ["manifest.json", "contract.json"] {
        files.insert(
            name.into(),
            read_bounded(&tool.directory.join(name), 1048576)?,
        );
    }
    files.insert(
        "module.wasm".into(),
        read_bounded(module, kortex_pack::MAX_FILE)?,
    );
    let icon = &tool.manifest.icon_path;
    if !icon.starts_with("assets/") || !kortex_pack::archive::safe_name(icon) {
        return Err("INVALID_ASSET_PATH".into());
    }
    let mut path = tool.directory.clone();
    for part in icon.split('/') {
        path.push(part);
        if link(&std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?) {
            return Err("SYMLINK_ASSET".into());
        }
    }
    files.insert(icon.clone(), read_bounded(&path, kortex_pack::MAX_FILE)?);
    let (manifest, _) =
        kortex_pack::validate::files(&files).map_err(|e| format!("{}:{}", e.code, e.path))?;
    if manifest.id != tool.manifest.id
        || manifest.version != tool.manifest.version
        || manifest.abi_version != tool.manifest.abi_version
    {
        return Err("SOURCE_METADATA_CHANGED_DURING_BUILD".into());
    }
    Ok(files)
}
pub fn package(
    root: &Path,
    tool: &Tool,
    out: &Path,
) -> Result<(PathBuf, kortex_pack::ValidationReport)> {
    let module = crate::build::build(root, tool)?;
    let bytes = kortex_pack::package(collect(tool, &module)?)
        .map_err(|e| format!("{}:{}", e.code, e.path))?;
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let path = out.join(format!(
        "{}-{}.tool",
        tool.manifest.id, tool.manifest.version
    ));
    if path.exists() {
        if read_bounded(&path, kortex_pack::MAX_ARCHIVE)? != bytes {
            return Err("OUTPUT_EXISTS_WITH_DIFFERENT_BYTES".into());
        }
    } else {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    let report = kortex_pack::inspect(&bytes);
    std::fs::write(
        path.with_extension("validation.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok((path, report))
}
