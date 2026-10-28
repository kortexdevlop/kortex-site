use crate::{config::Config, paths, trust};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Clone, Debug, PartialEq)]
pub struct Installed {
    pub id: String,
    pub version: String,
    pub name: String,
    pub archive: PathBuf,
    pub trust: PathBuf,
    pub trust_seal: String,
    pub sha256: String,
    pub permissions: Vec<String>,
    pub previous: Option<String>,
    pub versions: Vec<String>,
}
#[derive(Clone, Default)]
pub struct Library {
    pub tools: Vec<Installed>,
    pub rejected: usize,
    pub unavailable: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Version {
    pub sha256: String,
    pub blocked: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pending {
    pub previous: String,
    pub candidate: String,
    pub backup: String,
    pub had_data: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub current: String,
    pub previous: Option<String>,
    pub versions: BTreeMap<String, Version>,
    pub pending: Option<Pending>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Index {
    pub schema_version: u8,
    pub revision: u64,
    pub tools: BTreeMap<String, Entry>,
}
impl Default for Index {
    fn default() -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            tools: BTreeMap::new(),
        }
    }
}
pub fn validate(c: &Config, id: &str, version: &str) -> Result<Installed, String> {
    let folder = paths::version(&c.data_dir, id, version)?;
    let descriptor =
        kortex_verify::Descriptor::parse(&paths::read(&folder.join("package.tool.json"), 1048576)?)
            .map_err(|_| "Descripteur invalide.")?;
    let (bundle, bytes) = trust::select(c, &descriptor)?;
    let tool = kortex_runtime_native::verify_package_with_trust(
        paths::read(&folder.join("package.tool"), 25 * 1024 * 1024)?,
        &descriptor,
        &bundle,
        kortex_runtime_native::permissions::now_ms(),
    )
    .map_err(|_| "Signature, intégrité ou confiance du paquet refusée.")?;
    let m = tool.manifest();
    if m.id != id || m.version != version || !crate::versions::supported(&m.min_runtime)? {
        return Err("Cet outil nécessite une autre version de Kortex.".into());
    }
    let snapshot = trust::snapshot(c, &bytes)?;
    Ok(Installed {
        id: id.into(),
        version: version.into(),
        name: m.name.clone(),
        archive: folder.join("package.tool"),
        trust: snapshot,
        trust_seal: kortex_verify::sha256(&bytes),
        sha256: descriptor.sha256,
        permissions: m.permissions.clone(),
        previous: None,
        versions: vec![version.into()],
    })
}
pub fn load(c: &Config) -> Result<Index, String> {
    let path = c.data_dir.join("library.json");
    if !path.exists() {
        if c.data_dir.join("library-managed").exists() {
            return Err(
                "Index local manquant ; aucune version ne sera activée automatiquement.".into(),
            );
        }
        let mut index = Index::default();
        for i in legacy(c).tools {
            let e = index.tools.entry(i.id.clone()).or_default();
            e.versions.insert(
                i.version.clone(),
                Version {
                    sha256: i.sha256,
                    blocked: false,
                },
            );
            if e.current.is_empty()
                || semver::Version::parse(&i.version).unwrap()
                    > semver::Version::parse(&e.current).unwrap()
            {
                e.current = i.version;
            }
        }
        return Ok(index);
    }
    let value: Index = serde_json::from_slice(&paths::read(&path, 4 * 1024 * 1024)?)
        .map_err(|_| "Index local endommagé.")?;
    if value.schema_version != 1 || value.tools.len() > 10000 {
        return Err("Index local incompatible.".into());
    }
    for (id, e) in &value.tools {
        if !crate::api::slug(id)
            || e.versions.is_empty()
            || e.versions.len() > 64
            || !e.versions.contains_key(&e.current)
            || e.previous
                .as_ref()
                .is_some_and(|v| !e.versions.contains_key(v))
        {
            return Err("Index local incohérent.".into());
        }
        for (v, r) in &e.versions {
            paths::version(&c.data_dir, id, v)?;
            if r.sha256.len() != 64 || !r.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("Empreinte locale invalide.".into());
            }
        }
        if let Some(p) = &e.pending {
            if p.backup.len() != 32
                || !p.backup.bytes().all(|b| b.is_ascii_hexdigit())
                || p.candidate != e.current
                || !e.versions.contains_key(&p.previous)
            {
                return Err("Transaction locale incohérente.".into());
            }
        }
    }
    Ok(value)
}
pub fn save(c: &Config, index: &mut Index) -> Result<(), String> {
    index.revision = index.revision.checked_add(1).ok_or("Index saturé.")?;
    let bytes = serde_json::to_vec(index).map_err(|_| "Index invalide.")?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("Bibliothèque trop volumineuse.".into());
    }
    paths::atomic(&c.data_dir.join("library.json"), &bytes)?;
    if !c.data_dir.join("library-managed").exists() {
        paths::atomic(&c.data_dir.join("library-managed"), b"1")?
    }
    Ok(())
}
pub fn state_lock(c: &Config) -> Result<std::fs::File, String> {
    paths::lock(&c.data_dir, "library.lock", false)
}
pub fn lease(c: &Config, id: &str, shared: bool) -> Result<std::fs::File, String> {
    if !crate::api::slug(id) {
        return Err("Identifiant invalide.".into());
    }
    paths::lock(&c.data_dir.join("leases"), &format!("{id}.lock"), shared)
}
fn legacy(c: &Config) -> Library {
    let mut r = Library::default();
    let root = c.data_dir.join("installed");
    if !root.exists() {
        return r;
    }
    if paths::check(&root).is_err() {
        r.rejected = 1;
        return r;
    }
    if let Ok(dirs) = std::fs::read_dir(root) {
        for tool in dirs.flatten().take(10000) {
            let id = tool.file_name().to_string_lossy().into_owned();
            if paths::check(&tool.path()).is_err() {
                r.rejected += 1;
                continue;
            }
            if let Ok(versions) = std::fs::read_dir(tool.path()) {
                for v in versions.flatten().take(64) {
                    match validate(c, &id, &v.file_name().to_string_lossy()) {
                        Ok(i) => r.tools.push(i),
                        Err(_) => r.rejected += 1,
                    }
                }
            }
        }
    }
    r
}
pub fn scan(c: &Config) -> Library {
    if !c.data_dir.join("library.json").exists() {
        if c.data_dir.join("library-managed").exists() {
            return Library {
                tools: vec![],
                rejected: 1,
                unavailable: vec![],
            };
        }
        let all = legacy(c);
        let mut latest: BTreeMap<String, Installed> = BTreeMap::new();
        for i in all.tools {
            if latest.get(&i.id).is_none_or(|old| {
                semver::Version::parse(&i.version).unwrap()
                    > semver::Version::parse(&old.version).unwrap()
            }) {
                latest.insert(i.id.clone(), i);
            }
        }
        return Library {
            tools: latest.into_values().collect(),
            rejected: all.rejected,
            unavailable: vec![],
        };
    }
    let Ok(index) = load(c) else {
        return Library {
            tools: vec![],
            rejected: 1,
            unavailable: vec![],
        };
    };
    for (id, e) in &index.tools {
        if e.pending.is_some() && current(c, id).is_err() {
            if crate::updates::rollback(c, id).is_ok() {
                return scan(c);
            }
        }
    }
    let mut r = Library::default();
    for (id, e) in index.tools {
        let v = &e.versions[&e.current];
        if v.blocked {
            r.rejected += 1;
            r.unavailable.push(id);
            continue;
        }
        match validate(c, &id, &e.current) {
            Ok(mut i) if i.sha256 == v.sha256 => {
                i.previous = e.previous;
                i.versions = e.versions.keys().cloned().collect();
                r.tools.push(i)
            }
            _ => {
                r.rejected += 1;
                r.unavailable.push(id)
            }
        }
    }
    r.tools.sort_by(|a, b| a.name.cmp(&b.name));
    r
}
pub fn current(c: &Config, id: &str) -> Result<Installed, String> {
    let index = load(c)?;
    let e = index.tools.get(id).ok_or("Outil non installé.")?;
    let v = e.versions.get(&e.current).ok_or("Version non installée.")?;
    if v.blocked {
        return Err("Cette version est bloquée ou révoquée.".into());
    }
    let i = validate(c, id, &e.current)?;
    if i.sha256 != v.sha256 {
        return Err("Empreinte locale incohérente.".into());
    }
    Ok(i)
}
pub fn mark_blocked(c: &Config, id: &str, version: &str) -> Result<(), String> {
    let _lock = state_lock(c)?;
    let mut index = load(c)?;
    if let Some(v) = index
        .tools
        .get_mut(id)
        .and_then(|e| e.versions.get_mut(version))
    {
        v.blocked = true;
        save(c, &mut index)?
    }
    Ok(())
}

pub fn profile(c: &Config, id: &str) -> Result<PathBuf, String> {
    if !crate::api::slug(id) {
        return Err("Identifiant invalide.".into());
    }
    let storage = kortex_runtime_native::storage::NativeStorage::new(c.data_dir.join("tools"))
        .map_err(|_| "Données locales indisponibles.")?;
    storage
        .profile_file(&kortex_capabilities::storage::Owner {
            profile: "local".into(),
            tool_id: id.into(),
        })
        .map_err(|_| "Profil local indisponible.".into())
}
pub fn uninstall(c: &Config, id: &str, purge_data: bool) -> Result<(), String> {
    let _lock = state_lock(c)?;
    let _lease = lease(c, id, false)?;
    let mut index = load(c)?;
    let old = index.tools.remove(id).ok_or("Outil non installé.")?;
    if old.pending.is_some() {
        return Err("Validez ou annulez la mise à jour avant la désinstallation.".into());
    }
    let package = c.data_dir.join("installed").join(id);
    paths::check(&package)?;
    let trash = c.data_dir.join("trash");
    paths::dir(&trash)?;
    let transaction = trash.join(paths::nonce()?);
    paths::dir(&transaction)?;
    let data = if purge_data {
        Some(profile(c, id)?)
    } else {
        None
    };
    let _data_lock = if let Some(p) = &data {
        Some(paths::lock(
            p.parent().unwrap(),
            p.with_extension("lock")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            false,
        )?)
    } else {
        None
    };
    let mut moved_package = false;
    let mut moved_data = false;
    let result = (|| {
        if package.exists() {
            std::fs::rename(&package, transaction.join("package"))
                .map_err(|_| "Les fichiers de cet outil sont occupés.")?;
            moved_package = true
        }
        if let Some(p) = &data {
            if p.exists() {
                paths::check(p)?;
                std::fs::rename(p, transaction.join("profile.json"))
                    .map_err(|_| "Les données de cet outil sont occupées.")?;
                moved_data = true
            }
        }
        save(c, &mut index)
    })();
    if let Err(error) = result {
        if moved_data {
            std::fs::rename(transaction.join("profile.json"),data.as_ref().unwrap()).map_err(|_|"Restauration des données impossible ; sauvegarde conservée dans le dossier de récupération.")?
        }
        if moved_package {
            std::fs::rename(transaction.join("package"),&package).map_err(|_|"Restauration des fichiers impossible ; sauvegarde conservée dans le dossier de récupération.")?
        }
        let _ = paths::remove_tree(&trash, &transaction);
        return Err(error);
    }
    paths::remove_tree(&trash, &transaction)
}
