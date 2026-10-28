use std::path::PathBuf;
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub api_origin: String,
    pub channel: String,
    pub data_dir: PathBuf,
    pub trust_path: PathBuf,
    pub offline: bool,
}
impl Config {
    pub fn load() -> Result<Self, String> {
        let api = std::env::var("KORTEX_DESKTOP_API_ORIGIN")
            .unwrap_or_else(|_| "https://kortex-api-staging.userpgl.workers.dev".into());
        let channel = validate_origin(&api)?;
        let data_dir = std::env::var_os("KORTEX_DESKTOP_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| dirs::data_local_dir().map(|p| p.join("Kortex")))
            .ok_or("Dossier personnel indisponible.")?;
        let trust_path = std::env::var_os("KORTEX_DESKTOP_TRUST")
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("trust.json"));
        Ok(Self {
            api_origin: api.trim_end_matches('/').into(),
            channel,
            data_dir,
            trust_path,
            offline: std::env::var("KORTEX_DESKTOP_OFFLINE").as_deref() == Ok("1"),
        })
    }
}

pub fn validate_origin(api: &str) -> Result<String, String> {
    let parsed = url::Url::parse(&api).map_err(|_| "Adresse du catalogue invalide.")?;
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
        || parsed.path() != "/"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !matches!(
            parsed.host_str(),
            Some("kortex-api-staging.userpgl.workers.dev" | "api.kortex.site")
        )
    {
        return Err("Adresse du catalogue non autorisée.".into());
    }
    let channel: String = if parsed.host_str() == Some("api.kortex.site") {
        "production"
    } else {
        "staging"
    }
    .into();
    Ok(channel)
}
