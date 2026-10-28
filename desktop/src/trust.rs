use crate::{config::Config, paths};
use kortex_runtime_native::HostTrust;
use kortex_verify::Descriptor;
pub fn select(c: &Config, d: &Descriptor) -> Result<(HostTrust, Vec<u8>), String> {
    let candidates: Vec<HostTrust> = if c.trust_path.exists() {
        let bytes = paths::read(&c.trust_path, 1048576)?;
        if let Ok(one) = serde_json::from_slice::<HostTrust>(&bytes) {
            vec![one]
        } else {
            serde_json::from_slice(&bytes).map_err(|_| "Clés publiques hôte invalides.")?
        }
    } else if c.channel == "staging" {
        serde_json::from_str(include_str!("../assets/staging-trust.json"))
            .map_err(|_| "Clés de test indisponibles.")?
    } else {
        return Err("Les clés publiques de ce canal ne sont pas configurées.".into());
    };
    let mut matching = candidates
        .into_iter()
        .filter(|b| b.document.keys.iter().any(|k| k.id == d.signing_key_id));
    let bundle = matching.next().ok_or("Clé de signature inconnue.")?;
    if matching.next().is_some() {
        return Err("Configuration de confiance ambiguë.".into());
    }
    let bytes = serde_json::to_vec_pretty(&bundle).map_err(|_| "Clés publiques invalides.")?;
    Ok((bundle, bytes))
}
pub fn snapshot(c: &Config, bytes: &[u8]) -> Result<std::path::PathBuf, String> {
    let folder = c.data_dir.join("trust-cache");
    paths::dir(&folder)?;
    let path = folder.join(format!("{}.json", kortex_verify::sha256(bytes)));
    if path.exists() {
        if paths::read(&path, 1048576)? != bytes {
            return Err("Cache des clés incohérent.".into());
        }
    } else {
        paths::atomic(&path, bytes)?
    }
    Ok(path)
}
