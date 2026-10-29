use crate::{config::Config, models::*};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::Read,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
const LIMIT: u64 = 1024 * 1024;
const TTL: Duration = Duration::from_secs(300);
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Offline,
    NotFound,
    Unavailable,
    Invalid(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Offline => "Hors connexion. Votre bibliothèque reste disponible.",
                Self::NotFound => "Cet outil n’est plus disponible.",
                Self::Unavailable => "Catalogue indisponible. Réessayez dans un instant.",
                Self::Invalid(v) => v,
            }
        )
    }
}
#[derive(Clone)]
struct Cached {
    at: Instant,
    value: Value,
}
#[derive(Clone)]
pub struct Api {
    origin: String,
    client: reqwest::blocking::Client,
    cache: Arc<Mutex<HashMap<String, Cached>>>,
    offline: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    pub q: String,
    pub category: String,
    pub sort: String,
}
impl Query {
    pub fn params(&self, cursor: Option<&str>) -> Result<Vec<(String, String)>, Error> {
        if self.q.encode_utf16().count() > 200
            || self.q.chars().any(char::is_control)
            || (!self.category.is_empty() && !slug(&self.category))
            || !matches!(self.sort.as_str(), "" | "recent" | "name")
        {
            return Err(Error::Invalid("Recherche invalide.".into()));
        }
        if cursor.is_some_and(|c| c.is_empty() || c.len() > 4096) {
            return Err(Error::Invalid("Page invalide.".into()));
        }
        let mut pairs = vec![
            ("limit".into(), "24".into()),
            (
                "sort".into(),
                if self.sort.is_empty() {
                    "recent"
                } else {
                    &self.sort
                }
                .into(),
            ),
        ];
        if !self.q.trim().is_empty() {
            pairs.push(("q".into(), self.q.trim().into()))
        }
        if !self.category.is_empty() {
            pairs.push(("category".into(), self.category.clone()))
        }
        if let Some(c) = cursor {
            pairs.push(("cursor".into(), c.into()))
        }
        Ok(pairs)
    }
}
pub fn slug(s: &str) -> bool {
    (3..=64).contains(&s.len())
        && s.split('-').all(|v| {
            !v.is_empty()
                && v.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}
impl Api {
    // Retry one public read only when Windows aborts before response headers.
    // HTTP errors and body/signature failures are never retried here.
    fn send_read(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> Result<reqwest::blocking::Response, Error> {
        let retry = request.try_clone();
        match request.send() {
            Ok(value) => Ok(value),
            Err(error) => {
                let mut interrupted = false;
                #[cfg(windows)]
                {
                    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(&error);
                    while let Some(value) = cause {
                        if value
                            .downcast_ref::<std::io::Error>()
                            .is_some_and(|e| matches!(e.raw_os_error(), Some(10053 | 10054)))
                        {
                            interrupted = true;
                            break;
                        }
                        cause = value.source();
                    }
                }
                if interrupted {
                    if let Some(retry) = retry {
                        std::thread::sleep(Duration::from_millis(50));
                        return retry.send().map_err(|_| Error::Offline);
                    }
                }
                Err(Error::Offline)
            }
        }
    }
    pub fn offline_mode(&self, value: bool) -> Self {
        let mut api = self.clone();
        api.offline = value;
        api
    }
    pub fn new(c: &Config) -> Result<Self, Error> {
        crate::config::validate_origin(&c.api_origin).map_err(Error::Invalid)?;
        {
            let origin = c.api_origin.clone();
            let offline = c.offline;
            std::thread::spawn(move || Self::build(origin, offline))
                .join()
                .map_err(|_| Error::Unavailable)?
        }
    }
    fn build(origin: String, offline: bool) -> Result<Self, Error> {
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(12))
            .user_agent("KortexDesktop/1.0");
        // Local HTTP fixtures must not inherit a runner's outbound proxy or
        // reuse sockets from the deliberately interrupted one-shot server.
        #[cfg(debug_assertions)]
        let client = if origin.starts_with("http://127.0.0.1:") {
            client.no_proxy().pool_max_idle_per_host(0)
        } else {
            client
        };
        let client = client.build().map_err(|_| Error::Unavailable)?;
        Ok(Self {
            origin,
            client,
            offline,
            cache: Arc::default(),
        })
    }
    #[cfg(debug_assertions)]
    pub fn loopback_test(origin: &str) -> Self {
        let url = url::Url::parse(origin).unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("127.0.0.1"));
        Self::build(origin.into(), false).unwrap()
    }
    fn get(&self, path: &str, pairs: &[(String, String)], force: bool) -> Result<Value, Error> {
        if self.offline {
            return Err(Error::Offline);
        }
        let mut url =
            url::Url::parse(&(self.origin.clone() + path)).map_err(|_| Error::Unavailable)?;
        url.query_pairs_mut()
            .extend_pairs(pairs.iter().map(|(k, v)| (k, v)));
        let key = url.to_string();
        if !force {
            if let Some(v) = self
                .cache
                .lock()
                .unwrap()
                .get(&key)
                .filter(|v| v.at.elapsed() < TTL)
            {
                return Ok(v.value.clone());
            }
        }
        self.cache.lock().unwrap().remove(&key);
        let response = self.send_read(self.client.get(url))?;
        match response.status().as_u16() {
            200 => {}
            404 => {
                self.cache.lock().unwrap().remove(&key);
                return Err(Error::NotFound);
            }
            _ => return Err(Error::Unavailable),
        }
        if response.content_length().is_some_and(|v| v > LIMIT) {
            return Err(Error::Invalid("Réponse trop volumineuse.".into()));
        }
        let mut bytes = Vec::new();
        response
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::Unavailable)?;
        if bytes.len() as u64 > LIMIT {
            return Err(Error::Invalid("Réponse trop volumineuse.".into()));
        }
        let v: Value = serde_json::from_slice(&bytes)
            .map_err(|_| Error::Invalid("Réponse du catalogue invalide.".into()))?;
        // Cache only schema-validated values in the typed accessors.
        Ok(v)
    }
    fn remember(&self, path: &str, pairs: &[(String, String)], value: Value) {
        let mut url = url::Url::parse(&(self.origin.clone() + path)).unwrap();
        url.query_pairs_mut()
            .extend_pairs(pairs.iter().map(|(k, v)| (k, v)));
        let mut cache = self.cache.lock().unwrap();
        if cache
            .get(url.as_str())
            .is_some_and(|v| v.at.elapsed() < TTL)
        {
            return;
        }
        if cache.len() >= 128 {
            cache.clear()
        }
        cache.insert(
            url.to_string(),
            Cached {
                at: Instant::now(),
                value,
            },
        );
    }
    pub fn list(&self, q: &Query, cursor: Option<&str>, force: bool) -> Result<ToolList, Error> {
        let p = q.params(cursor)?;
        let v = self.get("/api/v1/tools", &p, force)?;
        let out = decode_list(v.clone()).map_err(Error::Invalid)?;
        self.remember("/api/v1/tools", &p, v);
        Ok(out)
    }
    pub fn detail(&self, id: &str) -> Result<ToolDetail, Error> {
        if !slug(id) {
            return Err(Error::Invalid("Identifiant invalide.".into()));
        }
        let path = format!("/api/v1/tools/{id}");
        let v = self.get(&path, &[], false)?;
        let out = decode_detail(v.clone(), id).map_err(Error::Invalid)?;
        self.remember(&path, &[], v);
        Ok(out)
    }

    pub fn runtime_channels(&self) -> Result<kortex_verify::runtime::Envelope, Error> {
        serde_json::from_value(self.get("/api/v1/runtime/channels", &[], true)?)
            .map_err(|_| Error::Invalid("Canaux Runtime invalides.".into()))
    }
    pub fn revocations(&self) -> Result<kortex_verify::revocation::Document, Error> {
        serde_json::from_value(self.get("/api/v1/trust/revocations", &[], true)?)
            .map_err(|_| Error::Invalid("Liste signée invalide.".into()))
    }
    pub fn detail_fresh(&self, id: &str) -> Result<ToolDetail, Error> {
        if !slug(id) {
            return Err(Error::Invalid("Identifiant invalide.".into()));
        }
        let v = self.get(&format!("/api/v1/tools/{id}"), &[], true)?;
        decode_detail(v, id).map_err(Error::Invalid)
    }
    pub fn actions_fresh(&self, id: &str, version: &str) -> Result<ToolActions, Error> {
        if !slug(id) {
            return Err(Error::Invalid("Identifiant invalide.".into()));
        }
        let v = self.get(&format!("/api/v1/tools/{id}/actions"), &[], true)?;
        decode_actions(v, id, version).map_err(Error::Invalid)
    }
    fn artifact_url(&self, d: &kortex_verify::Descriptor) -> Result<url::Url, Error> {
        if !slug(&d.tool_id) || d.version.len() > 128 || semver::Version::parse(&d.version).is_err()
        {
            return Err(Error::Invalid("Identité invalide.".into()));
        }
        let mut u = url::Url::parse(&self.origin).map_err(|_| Error::Unavailable)?;
        u.path_segments_mut()
            .map_err(|_| Error::Unavailable)?
            .extend(["api", "v1", "downloads", &d.tool_id, &d.version]);
        Ok(u)
    }
    pub fn download(
        &self,
        d: &kortex_verify::Descriptor,
    ) -> Result<reqwest::blocking::Response, Error> {
        if self.offline {
            return Err(Error::Offline);
        }
        let response = self.send_read(
            self.client
                .get(self.artifact_url(d)?)
                .timeout(Duration::from_secs(60)),
        )?;
        match response.status().as_u16() {
            200 => Ok(response),
            404 => Err(Error::NotFound),
            410 => Err(Error::Invalid("Cette version a été révoquée.".into())),
            _ => Err(Error::Unavailable),
        }
    }
    pub fn availability(&self, d: &kortex_verify::Descriptor) -> Result<bool, Error> {
        if self.offline {
            return Err(Error::Offline);
        }
        let response = self.send_read(self.client.head(self.artifact_url(d)?))?;
        match response.status().as_u16() {
            200 => {
                if let Some(hash) = response.headers().get("x-kortex-sha256") {
                    if hash.to_str().ok() != Some(d.sha256.as_str()) {
                        return Err(Error::Invalid("Empreinte distante incohérente.".into()));
                    }
                }
                Ok(true)
            }
            404 | 410 => Ok(false),
            _ => Err(Error::Unavailable),
        }
    }
    pub fn actions(&self, id: &str, version: &str) -> Result<ToolActions, Error> {
        if !slug(id) {
            return Err(Error::Invalid("Identifiant invalide.".into()));
        }
        let path = format!("/api/v1/tools/{id}/actions");
        let v = self.get(&path, &[], false)?;
        let out = decode_actions(v.clone(), id, version).map_err(Error::Invalid)?;
        self.remember(&path, &[], v);
        Ok(out)
    }
    #[cfg(debug_assertions)]
    pub fn expire_test_cache(&self) {
        for c in self.cache.lock().unwrap().values_mut() {
            c.at = Instant::now() - TTL - Duration::from_secs(1);
        }
    }
}
