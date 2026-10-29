use crate::{api::Api, config::Config, paths};
use kortex_verify::{Descriptor, keys::RootKey, revocation::Document};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cache {
    pub document: Document,
    pub seen: u64,
}
pub fn root(c: &Config) -> Result<RootKey, String> {
    let path = c.trust_path.with_file_name("revocation-root.json");
    let bytes = if path.exists() {
        paths::read(&path, 4096)?
    } else {
        if c.channel != "staging" {
            return Err("Racine de révocation production non configurée.".into());
        }
        include_bytes!("../../infra/trust/revocation-root.json").to_vec()
    };
    serde_json::from_slice(&bytes).map_err(|_| "Racine de révocation invalide.".into())
}
pub fn cache_path(c: &Config, r: &RootKey) -> std::path::PathBuf {
    c.data_dir.join("revocations").join(format!(
        "{}.json",
        kortex_verify::sha256(format!("{}|{}", c.api_origin, r.id).as_bytes())
    ))
}
pub fn check(c: &Config, api: &Api, p: &Descriptor) -> Result<(), String> {
    let root = root(c)?;
    let path = cache_path(c, &root);
    let now = kortex_runtime_native::permissions::now_ms();
    let old: Option<Cache> = if path.exists() {
        let v: Cache = serde_json::from_slice(&paths::read(&path, 1048576)?)
            .map_err(|_| "Cache de révocation invalide.")?;
        v.document
            .verify(&root)
            .map_err(|_| "Signature de révocation invalide.")?;
        Some(v)
    } else {
        None
    };
    if old.as_ref().is_some_and(|v| v.seen > now) {
        return Err("Horloge reculée ; vérification de sécurité requise.".into());
    }
    let next = if !c.offline {
        let d = api.revocations().map_err(|e| e.to_string())?;
        d.verify(&root)
            .map_err(|_| "Signature de révocation invalide.")?;
        if !d.fresh(now) || old.as_ref().is_some_and(|v| !d.successor(&v.document)) {
            return Err("Liste expirée ou rejouée ; reconnexion requise.".into());
        }
        d
    } else {
        old.as_ref()
            .ok_or("Reconnexion requise : aucune liste de révocation signée.")?
            .document
            .clone()
    };
    if !c.offline {
        paths::dir(path.parent().ok_or("Cache invalide.")?)?;
        paths::atomic(
            &path,
            &serde_json::to_vec(&Cache {
                document: next.clone(),
                seen: now,
            })
            .map_err(|_| "Cache invalide.")?,
        )?;
    }
    // Known revocation is permanent, even when the signed snapshot has expired.
    if next.revoked(p) {
        return Err("REVOKED".into());
    }
    if !next.fresh(now) {
        return Err("Reconnexion requise : la liste signée dépasse 24 heures.".into());
    }
    paths::dir(path.parent().ok_or("Cache invalide.")?)?;
    paths::atomic(
        &path,
        &serde_json::to_vec(&Cache {
            document: next,
            seen: now,
        })
        .map_err(|_| "Cache invalide.")?,
    )?;
    Ok(())
}
