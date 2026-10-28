use crate::{
    config::{Config, validate_origin},
    paths, secure_store,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use zeroize::Zeroize;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceSession {
    pub access_token: String,
    pub token_type: String,
    pub device_id: String,
    pub user_id: String,
    pub expires_at: u64,
    pub scopes: Vec<String>,
}
impl Drop for DeviceSession {
    fn drop(&mut self) {
        self.access_token.zeroize()
    }
}
#[derive(Clone, Default, PartialEq)]
pub struct View {
    pub connected: bool,
    pub user: String,
    pub enabled: bool,
    pub favorites: Vec<String>,
    pub preferences: Value,
    pub message: String,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    owner: String,
    enabled: bool,
    favorites: Vec<String>,
    preferences: Value,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn random() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| "Aléa sécurisé indisponible.")?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}
pub fn web_origin(c: &Config) -> Result<String, String> {
    validate_origin(&c.api_origin)?;
    Ok(if c.channel == "staging" {
        "https://kortex-web-staging.userpgl.workers.dev"
    } else {
        "https://kortex.site"
    }
    .into())
}
fn request(
    c: &Config,
    session: Option<&DeviceSession>,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
    etag: Option<&str>,
) -> Result<Value, String> {
    if c.offline {
        return Err("Hors connexion. Les outils locaux restent disponibles.".into());
    }
    validate_origin(&c.api_origin)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(25))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Connexion indisponible.")?;
    let mut request = client.request(method, format!("{}{path}", c.api_origin));
    if let Some(s) = session {
        if s.expires_at <= now() {
            return Err("SESSION_EXPIRED".into());
        }
        request = request
            .bearer_auth(&s.access_token)
            .header("X-Kortex-Account", &s.user_id)
    }
    if let Some(body) = body {
        request = request
            .header("Content-Type", "application/json")
            .body(serde_json::to_vec(&body).map_err(|_| "Requête invalide.")?)
    }
    if let Some(tag) = etag {
        request = request.header("If-Match", tag)
    }
    let response = request
        .send()
        .map_err(|_| "Compte momentanément injoignable.")?;
    let status = response.status();
    if status.as_u16() == 401 {
        return Err("SESSION_EXPIRED".into());
    }
    if status.as_u16() == 409 {
        return Err(
            "Les données ont changé sur un autre appareil. Actualisez avant de réessayer.".into(),
        );
    }
    if !status.is_success() {
        return Err("Opération du compte refusée. Vérifiez la synchronisation sur le Web.".into());
    }
    if status.as_u16() == 204 {
        return Ok(Value::Null);
    }
    let mut bytes = Vec::new();
    response
        .take(1048577)
        .read_to_end(&mut bytes)
        .map_err(|_| "Réponse interrompue.")?;
    if bytes.len() > 1048576 {
        return Err("Réponse trop volumineuse.".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "Réponse du compte invalide.".into())
}
pub struct PendingLogin {
    pub authorization_url: String,
    pub request_id: String,
    listener: TcpListener,
    verifier: String,
    state: String,
    redirect: String,
}
impl Drop for PendingLogin {
    fn drop(&mut self) {
        self.verifier.zeroize();
        self.state.zeroize()
    }
}
pub fn start(c: &Config) -> Result<PendingLogin, String> {
    if c.offline {
        return Err("Revenez en ligne pour connecter votre compte.".into());
    }
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|_| "Le retour du navigateur n’a pas pu être préparé.".to_string())?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "Retour navigateur indisponible.")?;
    let redirect = format!(
        "http://127.0.0.1:{}/callback",
        listener
            .local_addr()
            .map_err(|_| "Adresse locale indisponible.")?
            .port()
    );
    let verifier = random()?;
    let state = random()?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let response = request(
        c,
        None,
        reqwest::Method::POST,
        "/api/v1/auth/desktop/start",
        Some(
            json!({"challenge":challenge,"challenge_method":"S256","state":state,"redirect_uri":redirect}),
        ),
        None,
    )?;
    let authorization_url = response["authorization_url"]
        .as_str()
        .ok_or("Adresse de connexion invalide.")?
        .to_string();
    let request_id = response["request_id"]
        .as_str()
        .ok_or("Demande invalide.")?
        .to_string();
    if authorization_url
        != format!(
            "{}/account/connect-desktop?request_id={request_id}",
            web_origin(c)?
        )
    {
        return Err("Adresse de connexion refusée.".into());
    }
    Ok(PendingLogin {
        authorization_url,
        request_id,
        listener,
        verifier,
        state,
        redirect,
    })
}
pub fn callback_query(target: &str, state: &str) -> Result<String, String> {
    let url = url::Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| "Retour navigateur invalide.")?;
    if url.path() != "/callback" || url.query_pairs().count() != 2 {
        return Err("Retour navigateur invalide.".into());
    }
    let pairs: Vec<_> = url.query_pairs().collect();
    if pairs.iter().filter(|(k, _)| k == "state").count() != 1
        || pairs.iter().filter(|(k, _)| k == "code").count() != 1
    {
        return Err("Retour navigateur invalide.".into());
    }
    let returned = pairs
        .iter()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.as_ref())
        .unwrap_or("");
    if returned.len() != state.len()
        || returned
            .bytes()
            .zip(state.bytes())
            .fold(0u8, |a, (b, c)| a | (b ^ c))
            != 0
    {
        return Err("La demande de connexion ne correspond pas.".into());
    }
    let code = pairs
        .iter()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .to_string();
    if code.len() != 43
        || !code
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("Code invalide.".into());
    }
    Ok(code)
}
pub fn complete(
    c: &Config,
    pending: PendingLogin,
    cancel: Arc<AtomicBool>,
) -> Result<View, String> {
    let deadline = Instant::now() + Duration::from_secs(300);
    while Instant::now() < deadline && !cancel.load(Ordering::SeqCst) {
        match pending.listener.accept() {
            Ok((mut socket, address)) => {
                if !address.ip().is_loopback() {
                    continue;
                }
                socket.set_read_timeout(Some(Duration::from_secs(2))).ok();
                let mut bytes = [0u8; 4096];
                let size = socket.read(&mut bytes).unwrap_or(0);
                let text = std::str::from_utf8(&bytes[..size]).unwrap_or("");
                let mut lines = text.lines();
                let line = lines.next().unwrap_or("");
                let expected = url::Url::parse(&pending.redirect)
                    .map_err(|_| "Retour invalide.")?
                    .authority()
                    .to_owned();
                let host = text
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("Host: ")
                            .or_else(|| line.strip_prefix("host: "))
                    })
                    .unwrap_or("");
                let target = line
                    .strip_prefix("GET ")
                    .and_then(|s| s.strip_suffix(" HTTP/1.1"));
                let code = if host == expected {
                    target.and_then(|target| callback_query(target, &pending.state).ok())
                } else {
                    None
                };
                let Some(code) = code else {
                    let _=socket.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    continue;
                };
                if cancel.load(Ordering::SeqCst) {
                    return Err("Connexion annulée.".into());
                }
                let value = request(
                    c,
                    None,
                    reqwest::Method::POST,
                    "/api/v1/auth/desktop/token",
                    Some(
                        json!({"request_id":pending.request_id,"code":code,"verifier":pending.verifier}),
                    ),
                    None,
                )?;
                let session: DeviceSession =
                    serde_json::from_value(value).map_err(|_| "Session reçue invalide.")?;
                if !session.access_token.starts_with("kd_")
                    || session.access_token.len() != 46
                    || session.expires_at <= now()
                    || !session.scopes.iter().any(|s| s == "sync:read")
                {
                    return Err("Session reçue invalide.".into());
                }
                if cancel.load(Ordering::SeqCst) {
                    return Err("Connexion annulée.".into());
                }
                secure_store::save(c, &session)?;
                let content = "Connexion Kortex terminee. Vous pouvez revenir a l'application.";
                let _ = write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    content.len(),
                    content
                );
                return view(c);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100))
            }
            Err(_) => return Err("Le retour du navigateur a échoué.".into()),
        }
    }
    Err("Connexion annulée ou expirée.".into())
}
fn metadata_path(c: &Config) -> std::path::PathBuf {
    c.data_dir.join("account-metadata.json")
}
fn metadata(c: &Config, user: &str) -> Metadata {
    paths::read(&metadata_path(c), 262144)
        .ok()
        .and_then(|b| serde_json::from_slice::<Metadata>(&b).ok())
        .filter(|m| m.owner == user)
        .unwrap_or(Metadata {
            owner: user.into(),
            ..Default::default()
        })
}
fn save_metadata(c: &Config, data: &Metadata) -> Result<(), String> {
    paths::dir(&c.data_dir)?;
    paths::atomic(
        &metadata_path(c),
        &serde_json::to_vec(data).map_err(|_| "Préférences invalides.")?,
    )
}
pub fn view(c: &Config) -> Result<View, String> {
    let Some(session) = secure_store::load(c)? else {
        return Ok(View::default());
    };
    if session.expires_at <= now() {
        secure_store::clear(c)?;
        return Ok(View {
            message: "Session expirée. Reconnectez-vous.".into(),
            ..Default::default()
        });
    }
    let meta = metadata(c, &session.user_id);
    let mut result = View {
        connected: true,
        user: session.user_id.clone(),
        enabled: meta.enabled,
        favorites: meta.favorites,
        preferences: meta.preferences,
        message: String::new(),
    };
    if !c.offline {
        match request(
            c,
            Some(&session),
            reqwest::Method::GET,
            "/api/v1/me",
            None,
            None,
        ) {
            Ok(me) => {
                result.user = me["user"]["email"]
                    .as_str()
                    .unwrap_or(&session.user_id)
                    .to_string()
            }
            Err(error) if error == "SESSION_EXPIRED" => {
                secure_store::clear(c)?;
                return Ok(View {
                    message: "Session expirée ou révoquée. Reconnectez-vous.".into(),
                    ..Default::default()
                });
            }
            Err(error) => result.message = error,
        }
    }
    Ok(result)
}
fn saved(c: &Config) -> Result<DeviceSession, String> {
    secure_store::load(c)?.ok_or_else(|| "Connectez votre compte d’abord.".into())
}
pub fn synchronize(c: &Config, enable: Option<bool>) -> Result<View, String> {
    let session = saved(c)?;
    let mut meta = metadata(c, &session.user_id);
    let mut prefs = request(
        c,
        Some(&session),
        reqwest::Method::GET,
        "/api/v1/me/preferences",
        None,
        None,
    )?;
    if let Some(enabled) = enable {
        prefs = request(
            c,
            Some(&session),
            reqwest::Method::PATCH,
            "/api/v1/me/preferences",
            Some(json!({"sync_enabled":enabled})),
            Some(&format!("\"preferences-{}\"", prefs["revision"])),
        )?;
        meta.enabled = enabled
    }
    if !meta.enabled {
        save_metadata(c, &meta)?;
        return view(c);
    }
    if prefs["sync_enabled"] != true {
        meta.enabled = false;
        save_metadata(c, &meta)?;
        return Err("La synchronisation a été désactivée sur le Web.".into());
    }
    let library = request(
        c,
        Some(&session),
        reqwest::Method::GET,
        "/api/v1/me/library",
        None,
        None,
    )?;
    meta.favorites = library["items"]
        .as_array()
        .ok_or("Bibliothèque invalide.")?
        .iter()
        .filter_map(|v| v["tool_id"].as_str().map(String::from))
        .collect();
    if meta.favorites.len() > 2000 {
        return Err("Bibliothèque trop volumineuse.".into());
    }
    meta.preferences = prefs["preferences"].clone();
    save_metadata(c, &meta)?;
    view(c)
}
pub fn favorite(c: &Config, id: &str, add: bool) -> Result<View, String> {
    if id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("Outil invalide.".into());
    }
    let session = saved(c)?;
    if !metadata(c, &session.user_id).enabled {
        return Err("Activez la synchronisation sur cet appareil.".into());
    }
    let library = request(
        c,
        Some(&session),
        reqwest::Method::GET,
        "/api/v1/me/library",
        None,
        None,
    )?;
    let tags = library["items"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["tool_id"] == id))
        .map(|v| v["tags"].clone())
        .unwrap_or(json!([]));
    let operation = if add {
        let bytes = random()?;
        let digest = Sha256::digest(bytes.as_bytes());
        let hex = format!("{:x}", digest);
        let tag = format!(
            "{}-{}-4{}-8{}-{}",
            &hex[..8],
            &hex[8..12],
            &hex[13..16],
            &hex[17..20],
            &hex[20..32]
        );
        json!({"adds":[{"tool_id":id,"tag":tag}],"removes":[]})
    } else {
        if tags.as_array().is_none_or(|t| t.is_empty()) {
            return synchronize(c, None);
        }
        json!({"adds":[],"removes":[{"tool_id":id,"tags":tags}]})
    };
    request(
        c,
        Some(&session),
        reqwest::Method::PUT,
        "/api/v1/me/library",
        Some(operation),
        Some(&format!("\"library-{}\"", library["revision"])),
    )?;
    synchronize(c, None)
}
pub fn preference(c: &Config, key: &str, value: &str) -> Result<View, String> {
    let valid = match key {
        "theme" => ["light", "dark"].contains(&value),
        "density" => ["simple", "medium", "pro"].contains(&value),
        "type" => ["standard", "large"].contains(&value),
        "lang" => ["fr", "en"].contains(&value),
        _ => false,
    };
    if !valid {
        return Err("Réglage invalide.".into());
    }
    let session = saved(c)?;
    if !metadata(c, &session.user_id).enabled {
        return Err("Activez la synchronisation sur cet appareil.".into());
    }
    let prefs = request(
        c,
        Some(&session),
        reqwest::Method::GET,
        "/api/v1/me/preferences",
        None,
        None,
    )?;
    let mut changes = serde_json::Map::new();
    changes.insert(key.into(), Value::String(value.into()));
    request(
        c,
        Some(&session),
        reqwest::Method::PATCH,
        "/api/v1/me/preferences",
        Some(json!({"preferences":changes})),
        Some(&format!("\"preferences-{}\"", prefs["revision"])),
    )?;
    synchronize(c, None)
}
pub fn logout(c: &Config) -> Result<String, String> {
    let stored = secure_store::load(c)?;
    let revoked = if let Some(s) = stored {
        request(
            c,
            Some(&s),
            reqwest::Method::POST,
            "/api/v1/auth/desktop/logout",
            None,
            None,
        )
        .is_ok()
    } else {
        true
    };
    secure_store::clear(c)?;
    let path = metadata_path(c);
    paths::check(&path)?;
    if path.is_file() {
        std::fs::remove_file(path).map_err(|_| "Cache du compte non effacé.")?
    }
    Ok(if revoked {
        "Déconnecté. Les outils et fichiers locaux sont conservés."
    } else {
        "Session locale effacée. Si hors connexion, révoquez aussi cet appareil depuis le Web."
    }
    .into())
}
