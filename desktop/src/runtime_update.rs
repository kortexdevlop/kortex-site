use crate::{api::Api, config::Config, paths};
use kortex_verify::{
    keys::RootKey,
    runtime::{Desktop, Envelope},
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    envelope: Envelope,
    seen: u64,
}
pub fn check(c: &Config, api: &Api) -> Result<Option<Desktop>, String> {
    let root_text = match c.channel.as_str() {
        "staging" => include_str!("../../infra/trust/release-staging-root.json"),
        "production" => include_str!("../../infra/trust/release-production-root.json"),
        _ => return Err("Canal Runtime inconnu.".into()),
    };
    let root: RootKey = serde_json::from_str(root_text).map_err(|_| "Racine Runtime invalide.")?;
    let path = c.data_dir.join("runtime-channels").join(format!(
        "{}.json",
        kortex_verify::sha256(c.api_origin.as_bytes())
    ));
    let now = kortex_runtime_native::permissions::now_ms();
    let old: Option<Cache> = if path.exists() {
        let v: Cache = serde_json::from_slice(&paths::read(&path, 65536)?)
            .map_err(|_| "Cache Runtime invalide.")?;
        v.envelope
            .verify(&root, 1, v.seen)
            .map_err(|_| "Signature Runtime invalide.")?;
        if v.seen > now {
            return Err("Horloge reculée ; reconnectez Kortex.".into());
        }
        Some(v)
    } else {
        None
    };
    let envelope = if c.offline {
        old.as_ref()
            .ok_or("Reconnectez Kortex pour vérifier sa version.")?
            .envelope
            .clone()
    } else {
        api.runtime_channels().map_err(|e| e.to_string())?
    };
    envelope
        .verify(
            &root,
            old.as_ref().map_or(1, |v| v.envelope.document.sequence),
            now,
        )
        .map_err(|_| "Version Runtime expirée, rejouée ou signature invalide.")?;
    if old.as_ref().is_some_and(|v| {
        v.envelope.document.sequence == envelope.document.sequence
            && serde_json::to_value(&v.envelope.document).ok()
                != serde_json::to_value(&envelope.document).ok()
    }) {
        return Err("Version Runtime équivoque refusée.".into());
    }
    crate::revocation::check_signing_key(c, api, &envelope.document.signing_key_id)?;
    paths::dir(path.parent().ok_or("Cache Runtime invalide.")?)?;
    paths::atomic(
        &path,
        &serde_json::to_vec(&Cache {
            envelope: envelope.clone(),
            seen: now,
        })
        .map_err(|_| "Cache Runtime invalide.")?,
    )?;
    let channel = if c.channel == "production" {
        envelope.document.channels.stable
    } else {
        envelope.document.channels.staging
    };
    let Some(d) = channel.and_then(|r| r.desktop) else {
        return Ok(None);
    };
    if crate::versions::newer(&d.version, env!("CARGO_PKG_VERSION"))? {
        Ok(Some(d))
    } else {
        Ok(None)
    }
}
