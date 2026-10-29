use crate::{
    api::Api,
    config::Config,
    library::{self, Installed, Version},
    models::ToolDetail,
    paths, trust,
};
use std::{
    io::{Read, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
#[derive(Clone, Debug, Default)]
pub struct Progress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub phase: String,
}
pub type Monitor = Arc<Mutex<Progress>>;
pub fn install(
    c: &Config,
    api: &Api,
    detail: &ToolDetail,
    confirmed: &[String],
    cancel: &AtomicBool,
    progress: &Monitor,
    activate: bool,
) -> Result<Installed, String> {
    if confirmed != detail.manifest.permissions {
        return Err("Confirmez les permissions affichées avant l’installation.".into());
    }
    crate::revocation::check(c, api, &detail.descriptor)?;
    let fresh = api
        .detail_fresh(&detail.tool.id)
        .map_err(|e| e.to_string())?;
    if serde_json::to_value(&fresh).ok() != serde_json::to_value(detail).ok() {
        return Err("La fiche a changé. Consultez-la de nouveau avant d’installer.".into());
    }
    let _lock = library::state_lock(c)?;
    let _lease = library::lease(c, &detail.tool.id, false)?;
    let mut index = library::load(c)?;
    if !c.data_dir.join("library.json").exists() {
        library::save(c, &mut index)?
    }
    if let Some(e) = index.tools.get(&detail.tool.id) {
        if e.pending.is_some() {
            return Err("Une mise à jour attend déjà son premier démarrage.".into());
        }
        if activate && e.current != detail.tool.version {
            return Err(
                "Utilisez l’espace Mises à jour pour remplacer une version installée.".into(),
            );
        }
        if e.versions
            .get(&detail.tool.version)
            .is_some_and(|v| v.blocked)
        {
            return Err("Cette version est bloquée ou révoquée.".into());
        }
    }
    if index.tools.len() >= 10000 && !index.tools.contains_key(&detail.tool.id) {
        return Err("La bibliothèque a atteint sa limite.".into());
    }
    if index
        .tools
        .get(&detail.tool.id)
        .is_some_and(|e| e.versions.len() >= 64 && !e.versions.contains_key(&detail.tool.version))
    {
        return Err("Trop de versions conservées pour cet outil.".into());
    }
    let destination = paths::version(&c.data_dir, &detail.tool.id, &detail.tool.version)?;
    if destination.exists() {
        let installed = library::validate(c, &detail.tool.id, &detail.tool.version)?;
        if installed.sha256 != detail.descriptor.sha256 {
            return Err("Cette version existe avec un contenu différent.".into());
        }
        let e = index.tools.entry(installed.id.clone()).or_default();
        if !e.versions.contains_key(&installed.version) {
            e.versions.insert(
                installed.version.clone(),
                Version {
                    sha256: installed.sha256.clone(),
                    blocked: false,
                },
            );
            if e.current.is_empty() {
                e.current = installed.version.clone()
            }
            library::save(c, &mut index)?
        }
        return Ok(installed);
    }
    if !crate::versions::supported(&detail.manifest.min_runtime)? {
        return Err("Mettez Kortex à jour avant d’installer cet outil.".into());
    }
    if fs2::available_space(&c.data_dir).map_err(|_| "Espace disque indisponible.")?
        < 33 * 1024 * 1024
    {
        return Err("Espace disque insuffisant.".into());
    }
    let staging = c.data_dir.join("staging");
    paths::dir(&staging)?;
    let temporary = staging.join(paths::nonce()?);
    paths::dir(&temporary)?;
    let outcome = (|| {
        let mut response = api
            .download(&detail.descriptor)
            .map_err(|e| e.to_string())?;
        let total = response.content_length();
        if total.is_some_and(|n| n == 0 || n > 25 * 1024 * 1024) {
            return Err("Taille du téléchargement refusée.".into());
        }
        *progress.lock().unwrap() = Progress {
            downloaded: 0,
            total,
            phase: "Téléchargement".into(),
        };
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(temporary.join("package.tool"))
            .map_err(|_| "Écriture du téléchargement impossible.")?;
        let mut buffer = [0u8; 65536];
        let mut count = 0u64;
        loop {
            if cancel.load(Ordering::SeqCst) {
                return Err("Installation annulée.".into());
            }
            let n = response
                .read(&mut buffer)
                .map_err(|_| "Téléchargement interrompu. Réessayez pour repartir proprement.")?;
            if n == 0 {
                break;
            }
            count += n as u64;
            if count > 25 * 1024 * 1024 {
                return Err("Paquet trop volumineux.".into());
            }
            file.write_all(&buffer[..n])
                .map_err(|_| "Écriture impossible : vérifiez l’espace disque.")?;
            progress.lock().unwrap().downloaded = count;
        }
        if total.is_some_and(|n| n != count) {
            return Err("Téléchargement incomplet.".into());
        }
        file.sync_all()
            .map_err(|_| "Enregistrement du paquet impossible.")?;
        drop(file);
        if cancel.load(Ordering::SeqCst) {
            return Err("Installation annulée.".into());
        }
        progress.lock().unwrap().phase = "Vérification".into();
        let bytes = paths::read(&temporary.join("package.tool"), 25 * 1024 * 1024)?;
        let (bundle, public) = trust::select(c, &detail.descriptor)?;
        let verified = kortex_runtime_native::verify_package_with_trust(
            bytes,
            &detail.descriptor,
            &bundle,
            kortex_runtime_native::permissions::now_ms(),
        )
        .map_err(|_| "Paquet refusé : signature, intégrité ou archive invalide.")?;
        if serde_json::to_value(verified.manifest()).ok()
            != serde_json::to_value(&detail.manifest).ok()
            || serde_json::to_value(verified.contract()).ok()
                != serde_json::to_value(&detail.contract).ok()
        {
            return Err("Le contenu du paquet ne correspond pas à la fiche.".into());
        }
        trust::snapshot(c, &public)?;
        paths::atomic(
            &temporary.join("package.tool.json"),
            &serde_json::to_vec(&detail.descriptor).map_err(|_| "Descripteur invalide.")?,
        )?;
        paths::dir(destination.parent().unwrap())?;
        paths::check(&destination)?;
        std::fs::rename(&temporary, &destination).map_err(
            |_| "Installation atomique impossible ; aucun paquet incomplet n’a été activé.",
        )?;
        let installed = library::validate(c, &detail.tool.id, &detail.tool.version)?;
        let e = index.tools.entry(installed.id.clone()).or_default();
        e.versions.insert(
            installed.version.clone(),
            Version {
                sha256: installed.sha256.clone(),
                blocked: false,
            },
        );
        if e.current.is_empty() || activate {
            if !e.current.is_empty() && e.current != installed.version {
                e.previous = Some(e.current.clone())
            }
            e.current = installed.version.clone();
        }
        library::save(c, &mut index)?;
        progress.lock().unwrap().phase = "Installé".into();
        Ok(installed)
    })();
    if temporary.exists() {
        let _ = paths::remove_tree(&staging, &temporary);
    }
    outcome
}
