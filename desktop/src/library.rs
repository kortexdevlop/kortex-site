use crate::config::Config;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, PartialEq)]
pub struct Installed {
    pub id: String,
    pub version: String,
    pub name: String,
    pub archive: PathBuf,
    pub trust: PathBuf,
}
#[derive(Clone, Default)]
pub struct Library {
    pub tools: Vec<Installed>,
    pub rejected: usize,
}
fn linked(path: &Path) -> bool {
    let Ok(m) = std::fs::symlink_metadata(path) else {
        return true;
    };
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    m.file_type().is_symlink()
}
fn bounded(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    if linked(path)
        || std::fs::metadata(path)
            .map_err(|_| "Fichier illisible")?
            .len()
            > max
    {
        return Err("Fichier local refusé".into());
    }
    {
        use std::io::Read;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|_| "Fichier illisible")?
            .take(max + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Fichier illisible")?;
        if bytes.len() as u64 > max {
            return Err("Fichier local trop volumineux".into());
        }
        Ok(bytes)
    }
}
pub fn scan(config: &Config) -> Library {
    let mut result = Library::default();
    let root = config.data_dir.join("installed");
    if !root.exists() {
        return result;
    }
    if linked(&root) {
        result.rejected += 1;
        return result;
    }
    let trust = bounded(&config.trust_path, 1048576)
        .ok()
        .and_then(|b| serde_json::from_slice::<kortex_runtime_native::HostTrust>(&b).ok());
    let dirs = match std::fs::read_dir(&root) {
        Ok(v) => v,
        Err(_) => {
            result.rejected += 1;
            return result;
        }
    };
    for tool in dirs.flatten().take(256) {
        if linked(&tool.path()) || !tool.path().is_dir() {
            result.rejected += 1;
            continue;
        }
        let Ok(versions) = std::fs::read_dir(tool.path()) else {
            result.rejected += 1;
            continue;
        };
        for version in versions.flatten().take(64) {
            let folder = version.path();
            if linked(&folder) || !folder.is_dir() {
                result.rejected += 1;
                continue;
            }
            let checked = (|| {
                let trust = trust.as_ref().ok_or("Clés publiques hôte indisponibles")?;
                let proof = kortex_verify::Descriptor::parse(&bounded(
                    &folder.join("package.tool.json"),
                    1048576,
                )?)
                .map_err(|_| "Descripteur invalide")?;
                let archive = folder.join("package.tool");
                let verified = kortex_runtime_native::verify_package_with_trust(
                    bounded(&archive, 25 * 1024 * 1024)?,
                    &proof,
                    trust,
                    kortex_runtime_native::permissions::now_ms(),
                )
                .map_err(|_| "Paquet refusé")?;
                let m = verified.manifest();
                if tool.file_name().to_str() != Some(&m.id)
                    || version.file_name().to_str() != Some(&m.version)
                {
                    return Err("Identité locale incohérente".into());
                }
                Ok::<_, String>(Installed {
                    id: m.id.clone(),
                    version: m.version.clone(),
                    name: m.name.clone(),
                    archive,
                    trust: config.trust_path.clone(),
                })
            })();
            match checked {
                Ok(item) => result.tools.push(item),
                Err(_) => result.rejected += 1,
            }
        }
    }
    result
        .tools
        .sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
    result
}
