use crate::{Result, read_bounded};
use kortex_shared::{Manifest, validate_manifest};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[derive(Clone)]
pub struct Tool {
    pub directory: PathBuf,
    pub manifest: Manifest,
}
pub fn link(m: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    m.file_type().is_symlink()
}
pub fn discover(tools: &Path) -> Result<BTreeMap<String, Tool>> {
    if link(&std::fs::symlink_metadata(tools).map_err(|e| e.to_string())?) {
        return Err("SYMLINK_TOOL_ROOT".into());
    }
    let mut found = BTreeMap::new();
    for entry in std::fs::read_dir(tools).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if link(&metadata) {
            return Err("SYMLINK_TOOL".into());
        }
        if !metadata.is_dir() {
            continue;
        }
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let value = serde_json::from_slice(&read_bounded(&manifest_path, 1048576)?)
            .map_err(|e| e.to_string())?;
        let manifest = validate_manifest(&value).map_err(|_| "INVALID_MANIFEST")?;
        let id = manifest.id.clone();
        if found
            .insert(
                id,
                Tool {
                    directory: path,
                    manifest,
                },
            )
            .is_some()
        {
            return Err("DUPLICATE_TOOL_ID".into());
        }
    }
    Ok(found)
}
