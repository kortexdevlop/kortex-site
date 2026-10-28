use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub fn check(path: &Path) -> Result<(), String> {
    for part in path.ancestors() {
        match fs::symlink_metadata(part) {
            Ok(m) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if m.file_attributes() & 0x400 != 0 {
                        return Err("Lien de dossier refusé.".into());
                    }
                }
                if m.file_type().is_symlink() {
                    return Err("Lien de fichier refusé.".into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Dossier local indisponible.".into()),
        }
    }
    Ok(())
}
pub fn dir(path: &Path) -> Result<(), String> {
    check(path)?;
    fs::create_dir_all(path).map_err(|_| "Impossible de créer le dossier local.")?;
    check(path)
}
pub fn read(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    check(path)?;
    let mut f = File::open(path).map_err(|_| "Fichier local illisible.")?;
    if !f
        .metadata()
        .map_err(|_| "Fichier local illisible.")?
        .is_file()
    {
        return Err("Fichier local refusé.".into());
    }
    let mut b = Vec::new();
    Read::by_ref(&mut f)
        .take(max + 1)
        .read_to_end(&mut b)
        .map_err(|_| "Fichier local illisible.")?;
    if b.len() as u64 > max {
        return Err("Fichier local trop volumineux.".into());
    }
    Ok(b)
}
pub fn nonce() -> Result<String, String> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|_| "Création du fichier temporaire impossible.")?;
    Ok(b.iter().map(|v| format!("{v:02x}")).collect())
}
pub fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Dossier invalide.")?;
    dir(parent)?;
    let temp = parent.join(format!(".{}.tmp", nonce()?));
    let result = (|| {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "Écriture impossible.")?;
        f.write_all(bytes)
            .and_then(|_| f.sync_all())
            .map_err(|_| "Écriture impossible : vérifiez l’espace disque.")?;
        drop(f);
        check(path)?;
        fs::rename(&temp, path).map_err(|_| "Enregistrement local impossible.".to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
pub fn lock(root: &Path, name: &str, shared: bool) -> Result<File, String> {
    dir(root)?;
    if !name
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
    {
        return Err("Verrou invalide.".into());
    }
    let path = root.join(name);
    check(&path)?;
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| "Verrou local indisponible.")?;
    if shared {
        fs2::FileExt::try_lock_shared(&f)
    } else {
        fs2::FileExt::try_lock_exclusive(&f)
    }
    .map_err(|_| "Cet outil ou une opération est déjà en cours. Fermez l’outil puis réessayez.")?;
    Ok(f)
}
pub fn version(root: &Path, id: &str, version: &str) -> Result<PathBuf, String> {
    if !crate::api::slug(id) || version.len() > 128 || semver::Version::parse(version).is_err() {
        return Err("Identité de l’outil invalide.".into());
    }
    let p = root.join("installed").join(id).join(version);
    check(&p)?;
    Ok(p)
}
pub fn remove_tree(root: &Path, target: &Path) -> Result<(), String> {
    check(root)?;
    if !target.exists() {
        return Ok(());
    }
    check(target)?;
    let base = root.canonicalize().map_err(|_| "Dossier invalide.")?;
    let exact = target.canonicalize().map_err(|_| "Dossier invalide.")?;
    if exact == base || !exact.starts_with(&base) {
        return Err("Suppression hors du dossier autorisé refusée.".into());
    }
    fn remove(p: &Path, base: &Path) -> Result<(), String> {
        check(p)?;
        let absolute = p.canonicalize().map_err(|_| "Dossier invalide.")?;
        if absolute == base || !absolute.starts_with(base) {
            return Err("Suppression refusée.".into());
        }
        let m = fs::symlink_metadata(p).map_err(|_| "Fichier occupé.")?;
        if m.is_dir() {
            for e in fs::read_dir(p).map_err(|_| "Dossier occupé.")? {
                remove(&e.map_err(|_| "Fichier occupé.")?.path(), base)?
            }
            fs::remove_dir(p).map_err(|_| "Dossier occupé.".into())
        } else {
            fs::remove_file(p).map_err(|_| "Fichier occupé.".into())
        }
    }
    remove(&exact, &base)
}
