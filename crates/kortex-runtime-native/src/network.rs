use base64::Engine;
use kortex_capabilities::{CapabilityError, CapabilityResponse};
use serde_json::json;
use std::{
    collections::BTreeSet,
    io::Read,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    time::Duration,
};
#[derive(Clone, Default)]
pub struct NativeNetwork {
    allowed: BTreeSet<String>,
}
impl NativeNetwork {
    pub fn new(origins: impl IntoIterator<Item = String>) -> Result<Self, CapabilityError> {
        let mut allowed = BTreeSet::new();
        for s in origins {
            let u = Self::url(&s)?;
            if u.origin().ascii_serialization() != s {
                return Err(CapabilityError::NetworkDenied);
            }
            allowed.insert(s);
        }
        Ok(Self { allowed })
    }
    pub fn url(value: &str) -> Result<url::Url, CapabilityError> {
        let u = url::Url::parse(value).map_err(|_| CapabilityError::NetworkDenied)?;
        if u.scheme() != "https"
            || u.host_str().is_none()
            || !u.username().is_empty()
            || u.password().is_some()
            || u.fragment().is_some()
        {
            return Err(CapabilityError::NetworkDenied);
        }
        Ok(u)
    }
    pub fn resolve(host: &str) -> Result<Vec<IpAddr>, CapabilityError> {
        let host = host.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("kortex-dns".into())
            .spawn(move || {
                let result = (host.as_str(), 443)
                    .to_socket_addrs()
                    .map(|v| v.map(|a| a.ip()).collect())
                    .map_err(|_| CapabilityError::NetworkDenied);
                let _ = tx.send(result);
            })
            .map_err(|_| CapabilityError::HostFailure)?;
        rx.recv_timeout(Duration::from_secs(10))
            .map_err(|_| CapabilityError::NetworkDenied)?
    }
    pub fn fetch(
        &self,
        url: &str,
        addresses: &[IpAddr],
        remaining: Duration,
    ) -> Result<CapabilityResponse, CapabilityError> {
        let u = Self::url(url)?;
        if !self.allowed.contains(&u.origin().ascii_serialization()) || addresses.is_empty() {
            return Err(CapabilityError::NetworkDenied);
        }
        let sockets = addresses
            .iter()
            .map(|ip| SocketAddr::new(*ip, u.port_or_known_default().unwrap_or(443)))
            .collect::<Vec<_>>();
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .resolve_to_addrs(
                u.host_str().ok_or(CapabilityError::NetworkDenied)?,
                &sockets,
            )
            .timeout(remaining.min(Duration::from_secs(30)))
            .build()
            .map_err(|_| CapabilityError::HostFailure)?;
        let response = client
            .get(u)
            .send()
            .map_err(|_| CapabilityError::HostFailure)?;
        if response.status().is_redirection() {
            return Ok(CapabilityResponse::Redirect(
                response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or(CapabilityError::NetworkDenied)?
                    .into(),
            ));
        }
        if !response.status().is_success() {
            return Err(CapabilityError::HostFailure);
        }
        let status = response.status().as_u16();
        let mut body = Vec::new();
        response
            .take(256 * 1024 + 1)
            .read_to_end(&mut body)
            .map_err(|_| CapabilityError::HostFailure)?;
        if body.len() > 256 * 1024 {
            return Err(CapabilityError::SizeLimit);
        }
        Ok(CapabilityResponse::Bytes(serde_json::to_vec(&json!({"status":status,"data_base64":base64::engine::general_purpose::STANDARD.encode(body)})).map_err(|_|CapabilityError::HostFailure)?))
    }
}
