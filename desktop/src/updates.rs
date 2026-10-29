use crate::{
    api::Api,
    config::Config,
    installer::{self, Monitor},
    launcher,
    library::{self, Installed, Pending},
    models::ToolDetail,
    paths,
};
use std::sync::atomic::AtomicBool;
#[derive(Clone, Debug)]
pub struct Update {
    pub installed: Installed,
    pub detail: ToolDetail,
    pub extra_permissions: Vec<String>,
}
pub fn check(c: &Config, api: &Api) -> Result<Vec<Update>, String> {
    let index = library::load(c)?;
    let mut out = Vec::new();
    for (id, e) in &index.tools {
        let i = library::current(c, id)?;
        let descriptor = kortex_verify::Descriptor::parse(&paths::read(
            &i.archive.with_extension("tool.json"),
            1048576,
        )?)
        .map_err(|_| "Descripteur invalide.")?;
        crate::revocation::check(c, api, &descriptor)?;
        if !api.availability(&descriptor).map_err(|e| e.to_string())? {
            library::mark_blocked(c, id, &i.version)?;
            return Err(format!("{} : la version installée est révoquée.", i.name));
        }
        let d = api.detail_fresh(id).map_err(|e| e.to_string())?;
        if e.pending.is_some() {
            continue;
        }
        if crate::versions::newer(&d.tool.version, &i.version)? {
            let extra_permissions = d
                .manifest
                .permissions
                .iter()
                .filter(|p| !i.permissions.contains(p))
                .cloned()
                .collect();
            out.push(Update {
                installed: i,
                detail: d,
                extra_permissions,
            });
        }
    }
    Ok(out)
}
fn backup_path(c: &Config, id: &str, nonce: &str) -> Result<std::path::PathBuf, String> {
    if !crate::api::slug(id) || nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Sauvegarde invalide.".into());
    }
    let p = c
        .data_dir
        .join("rollback")
        .join(format!("{id}-{nonce}.json"));
    paths::check(&p)?;
    Ok(p)
}
pub fn apply(
    c: &Config,
    api: &Api,
    u: &Update,
    confirmed: &[String],
    cancel: &AtomicBool,
    monitor: &Monitor,
) -> Result<Installed, String> {
    apply_checked(c, api, u, confirmed, cancel, monitor, launcher::probe)
}
pub fn apply_checked<F: FnOnce(&Installed) -> Result<(), String>>(
    c: &Config,
    api: &Api,
    u: &Update,
    confirmed: &[String],
    cancel: &AtomicBool,
    monitor: &Monitor,
    check: F,
) -> Result<Installed, String> {
    let current = library::current(c, &u.installed.id)?;
    if current.version != u.installed.version || current.sha256 != u.installed.sha256 {
        return Err("La version installée a changé ; relancez la recherche.".into());
    }
    let next = installer::install(c, api, &u.detail, confirmed, cancel, monitor, false)?;
    let _lock = library::state_lock(c)?;
    let _lease = library::lease(c, &next.id, false)?;
    let mut index = library::load(c)?;
    let e = index.tools.get_mut(&next.id).ok_or("Outil non installé.")?;
    if e.current != current.version || e.pending.is_some() {
        return Err("Une autre mise à jour est en cours.".into());
    }
    let profile = library::profile(c, &next.id)?;
    let _data_lock = paths::lock(
        profile.parent().unwrap(),
        profile
            .with_extension("lock")
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        false,
    )?;
    let backup = paths::nonce()?;
    let path = backup_path(c, &next.id, &backup)?;
    let had_data = profile.exists();
    let data = if had_data {
        paths::read(&profile, 24 * 1024 * 1024)?
    } else {
        Vec::new()
    };
    paths::atomic(&path, &data)?;
    monitor.lock().unwrap().phase = "Validation de la nouvelle version".into();
    let checked = check(&next).and_then(|()| {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            Err("Mise à jour annulée.".into())
        } else {
            Ok(())
        }
    });
    if let Err(error) = checked {
        if had_data {
            paths::atomic(&profile, &data)?
        } else if profile.exists() {
            paths::check(&profile)?;
            std::fs::remove_file(&profile)
                .map_err(|_| "Restauration des préférences impossible.")?
        }
        let _ = std::fs::remove_file(path);
        return Err(format!(
            "Mise à jour annulée, ancienne version conservée : {error}"
        ));
    }
    e.previous = Some(current.version.clone());
    e.current = next.version.clone();
    e.pending = Some(Pending {
        previous: current.version,
        candidate: next.version.clone(),
        backup,
        had_data,
    });
    if let Err(error) = library::save(c, &mut index) {
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
    Ok(next)
}
pub fn confirm_launch(c: &Config, id: &str, version: &str) -> Result<(), String> {
    let _lock = library::state_lock(c)?;
    let mut index = library::load(c)?;
    let e = index.tools.get_mut(id).ok_or("Outil non installé.")?;
    if e.current != version {
        return Err("La version active a changé.".into());
    }
    if let Some(p) = e.pending.take() {
        if p.candidate != version {
            return Err("Transaction incohérente.".into());
        }
        library::save(c, &mut index)?;
        let _ = std::fs::remove_file(backup_path(c, id, &p.backup)?);
    }
    Ok(())
}
pub fn rollback(c: &Config, id: &str) -> Result<Installed, String> {
    let _lock = library::state_lock(c)?;
    let _lease = library::lease(c, id, false)?;
    let mut index = library::load(c)?;
    let e = index.tools.get_mut(id).ok_or("Outil non installé.")?;
    let previous = e
        .pending
        .as_ref()
        .map(|p| p.previous.clone())
        .or_else(|| e.previous.clone())
        .ok_or("Aucune version précédente disponible.")?;
    if e.versions.get(&previous).is_none_or(|v| v.blocked) {
        return Err("La version précédente est bloquée.".into());
    }
    let old = library::validate(c, id, &previous)?;
    if old.sha256 != e.versions[&previous].sha256 {
        return Err("Empreinte de la version précédente incohérente.".into());
    }
    let pending = e.pending.take();
    let failed = e.current.clone();
    if let Some(p) = &pending {
        let profile = library::profile(c, id)?;
        let _data_lock = paths::lock(
            profile.parent().unwrap(),
            profile
                .with_extension("lock")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            false,
        )?;
        if p.had_data {
            paths::atomic(
                &profile,
                &paths::read(&backup_path(c, id, &p.backup)?, 24 * 1024 * 1024)?,
            )?
        } else if profile.exists() {
            paths::check(&profile)?;
            std::fs::remove_file(profile).map_err(|_| "Restauration des données impossible.")?
        }
    }
    e.current = previous;
    e.previous = None;
    if pending.is_some() {
        if let Some(v) = e.versions.get_mut(&failed) {
            v.blocked = true
        }
    }
    library::save(c, &mut index)?;
    if let Some(p) = pending {
        let _ = std::fs::remove_file(backup_path(c, id, &p.backup)?);
    }
    Ok(old)
}
