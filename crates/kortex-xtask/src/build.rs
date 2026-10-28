use crate::{Result, discovery::Tool};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
fn cargo() -> std::ffi::OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}
/// Resolve package/target names from Cargo metadata, never from an identifier match.
pub fn artifact(root: &Path, tool: &Tool) -> Result<(String, String, PathBuf)> {
    let output = Command::new(cargo())
        .current_dir(&tool.directory)
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .env("CARGO_TARGET_DIR", root.join("target"))
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("CARGO_METADATA_FAILED".into());
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let manifest = tool
        .directory
        .join("Cargo.toml")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("INVALID_CARGO_METADATA")?;
    let package = packages
        .iter()
        .find(|p| {
            Path::new(p["manifest_path"].as_str().unwrap_or(""))
                .canonicalize()
                .ok()
                .as_ref()
                == Some(&manifest)
        })
        .ok_or("TOOL_NOT_IN_WORKSPACE")?;
    let targets = package["targets"].as_array().ok_or("NO_TARGET")?;
    let targets: Vec<_> = targets
        .iter()
        .filter(|t| {
            t["crate_types"]
                .as_array()
                .is_some_and(|k| k.iter().any(|v| v == "cdylib"))
        })
        .collect();
    if targets.len() != 1 {
        return Err("EXPECTED_ONE_CDYLIB".into());
    }
    let name = package["name"]
        .as_str()
        .ok_or("NO_PACKAGE_NAME")?
        .to_string();
    let target = targets[0]["name"]
        .as_str()
        .ok_or("NO_TARGET_NAME")?
        .replace('-', "_");
    let target_dir = PathBuf::from(
        metadata["target_directory"]
            .as_str()
            .ok_or("NO_TARGET_DIRECTORY")?,
    );
    Ok((
        name,
        target.clone(),
        target_dir
            .join("wasm32-unknown-unknown/release")
            .join(format!("{target}.wasm")),
    ))
}
pub fn build(root: &Path, tool: &Tool) -> Result<PathBuf> {
    let (name, _, path) = artifact(root, tool)?;
    let flags = format!(
        "{} -C link-arg=--max-memory=67108864",
        std::env::var("RUSTFLAGS").unwrap_or_default()
    );
    let status = Command::new(cargo())
        .current_dir(&tool.directory)
        .args([
            "build",
            "-p",
            &name,
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--locked",
        ])
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTFLAGS", flags.trim())
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("BUILD_FAILED".into());
    }
    kortex_pack::validate::wasm(&crate::read_bounded(&path, kortex_pack::MAX_FILE)?)
        .map_err(|e| format!("{}:{}", e.code, e.path))?;
    Ok(path)
}
