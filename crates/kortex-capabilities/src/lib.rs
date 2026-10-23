#![allow(async_fn_in_trait)]
pub mod audit;
pub mod handles;
pub mod policy;
pub mod storage;
use audit::{AuditEvent, AuditLog};
use handles::Handles;
use policy::{Context, Decision, GestureToken, Policy};
use std::net::{IpAddr, Ipv4Addr};
pub use storage::StorageBackend;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Capability {
    FilePick,
    FileRead,
    FileSave,
    StorageRead,
    StorageWrite,
    NetworkFetch,
    ClipboardWrite,
}
impl Capability {
    pub fn needs_gesture(self) -> bool {
        matches!(self, Self::FilePick | Self::FileSave | Self::ClipboardWrite)
    }
}
impl TryFrom<&str> for Capability {
    type Error = CapabilityError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "file.pick" => Ok(Self::FilePick),
            "file.read" => Ok(Self::FileRead),
            "file.save" => Ok(Self::FileSave),
            "storage.read" => Ok(Self::StorageRead),
            "storage.write" => Ok(Self::StorageWrite),
            "network.fetch" => Ok(Self::NetworkFetch),
            "clipboard.write" => Ok(Self::ClipboardWrite),
            _ => Err(CapabilityError::Denied),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resource {
    File { handle: String },
    FileDialog,
    Storage { key: String },
    Network { url: String },
    Clipboard,
}
impl Resource {
    pub fn scope(&self, capability: Capability) -> Result<String, CapabilityError> {
        match (self, capability) {
            (Self::File { handle }, Capability::FileRead | Capability::FileSave)
                if handle.len() == 64 && handle.bytes().all(|b| b.is_ascii_hexdigit()) =>
            {
                Ok(format!("file:{handle}"))
            }
            (Self::FileDialog, Capability::FilePick | Capability::FileSave) => {
                Ok("file-dialog".into())
            }
            (Self::Storage { key }, Capability::StorageRead | Capability::StorageWrite)
                if !key.is_empty()
                    && key.len() <= 256
                    && key
                        .split('/')
                        .all(|part| !part.is_empty() && part != "." && part != "..")
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b)) =>
            {
                Ok(format!("storage:{key}"))
            }
            (Self::Network { url }, Capability::NetworkFetch) => {
                Ok(network_url(url)?.origin().ascii_serialization())
            }
            (Self::Clipboard, Capability::ClipboardWrite) => Ok("clipboard".into()),
            _ => Err(CapabilityError::Denied),
        }
    }
}
fn network_url(value: &str) -> Result<url::Url, CapabilityError> {
    let url = url::Url::parse(value).map_err(|_| CapabilityError::NetworkDenied)?;
    if !["http", "https"].contains(&url.scheme())
        || url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(CapabilityError::NetworkDenied);
    }
    Ok(url)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    Denied,
    NeedsConsent,
    InvalidHandle,
    InvalidRequest,
    SizeLimit,
    NetworkDenied,
    RedirectLimit,
    Unsupported,
    HostFailure,
    Entropy,
}
pub struct CapabilityCall {
    pub request_id: String,
    pub capability: Capability,
    pub resource: Resource,
    pub bytes: Vec<u8>,
    pub max_output_bytes: usize,
}
#[derive(Debug)]
pub enum CapabilityResponse {
    Bytes(Vec<u8>),
    Redirect(String),
}
pub struct AuthorizedCall<'a> {
    context: &'a Context,
    request: &'a CapabilityCall,
    addresses: Vec<IpAddr>,
    backend: Option<String>,
    expires_ms: u64,
}
impl AuthorizedCall<'_> {
    pub fn expires_ms(&self) -> u64 {
        self.expires_ms
    }
    pub fn context(&self) -> &Context {
        self.context
    }
    pub fn request(&self) -> &CapabilityCall {
        self.request
    }
    pub fn addresses(&self) -> &[IpAddr] {
        &self.addresses
    }
    pub fn file_backend(&self) -> Option<&str> {
        self.backend.as_deref()
    }
}
/// Trusted host adapter: use only the validated resource, never auto-follow redirects,
/// pin connections to the approved addresses, and return Unsupported if this cannot be enforced.
/// Storage must be namespaced by context.tool_id, never by a key supplied in bytes.
pub trait CapabilityHost {
    async fn resolve(&mut self, _host: &str) -> Result<Vec<IpAddr>, CapabilityError> {
        Err(CapabilityError::Unsupported)
    }
    async fn execute(
        &mut self,
        call: &AuthorizedCall<'_>,
    ) -> Result<CapabilityResponse, CapabilityError>;
}
fn public_v4(ip: Ipv4Addr) -> bool {
    let a = ip.octets();
    !ip.is_private()
        && !ip.is_loopback()
        && !ip.is_link_local()
        && !ip.is_broadcast()
        && !ip.is_documentation()
        && !ip.is_unspecified()
        && !ip.is_multicast()
        && a[0] != 0
        && a[0] < 224
        && !(a[0] == 100 && (64..=127).contains(&a[1]))
        && !(a[0] == 192 && a[1] == 0 && a[2] == 0)
        && !(a[0] == 198 && (18..=19).contains(&a[1]))
}
fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => public_v4(ip),
        IpAddr::V6(ip) => {
            if let Some(ip) = ip.to_ipv4_mapped() {
                return public_v4(ip);
            }
            let s = ip.segments();
            s[0] & 0xe000 == 0x2000
                && !(s[0] == 0x2001 && (s[1] == 0xdb8 || s[1] < 0x200))
                && s[0] != 0x2002
                && !ip.is_multicast()
        }
    }
}
pub struct ConsentPrompt {
    pub tool_id: String,
    pub version: String,
    pub instance_id: u64,
    pub capability: Capability,
    pub scope: String,
    pub duration: policy::Duration,
    pub expires_ms: u64,
}
pub struct Gateway<H: CapabilityHost> {
    pub policy: Policy,
    pub handles: Handles,
    pub audit: AuditLog,
    host: H,
    file_limit: usize,
}
impl<H: CapabilityHost> Gateway<H> {
    pub fn new(host: H) -> Self {
        Self {
            policy: Policy::default(),
            handles: Handles::default(),
            audit: AuditLog::default(),
            host,
            file_limit: 10 * 1024 * 1024,
        }
    }
    /// Description for trusted local consent UI. Never store this in the audit journal.
    pub fn consent_prompt(
        &self,
        context: &Context,
        capability: Capability,
        resource: &Resource,
        duration: policy::Duration,
        expires_ms: u64,
    ) -> Result<ConsentPrompt, CapabilityError> {
        if !self.policy.is_declared(context, capability) {
            return Err(CapabilityError::Denied);
        }
        Ok(ConsentPrompt {
            tool_id: context.tool_id.clone(),
            version: context.version.clone(),
            instance_id: context.instance_id,
            capability,
            scope: resource.scope(capability)?,
            duration,
            expires_ms,
        })
    }
    pub fn set_file_limit(&mut self, bytes: usize) -> Result<(), CapabilityError> {
        if bytes == 0 || bytes > 10 * 1024 * 1024 {
            return Err(CapabilityError::SizeLimit);
        }
        self.file_limit = bytes;
        Ok(())
    }
    pub fn revoke_instance(&mut self, context: &Context) {
        self.policy.revoke_instance(context);
        self.handles.revoke_instance(context);
    }
    pub async fn execute(
        &mut self,
        context: &Context,
        request: CapabilityCall,
        gesture: Option<&GestureToken>,
        time: &impl Fn() -> u64,
    ) -> Result<CapabilityResponse, CapabilityError> {
        let bytes = request.bytes.len();
        let capability = request.capability;
        let result = self.execute_inner(context, request, gesture, time).await;
        self.audit.record(AuditEvent {
            at_ms: time(),
            instance_id: context.instance_id,
            capability,
            outcome: audit::outcome(&result),
            bytes_in: bytes,
        });
        result
    }
    async fn execute_inner(
        &mut self,
        context: &Context,
        mut request: CapabilityCall,
        gesture: Option<&GestureToken>,
        time: &impl Fn() -> u64,
    ) -> Result<CapabilityResponse, CapabilityError> {
        let cap = request.capability;
        let limit = if matches!(
            cap,
            Capability::FileRead | Capability::FileSave | Capability::FilePick
        ) {
            self.file_limit
        } else {
            kortex_abi::MAX_MESSAGE_BYTES
        };
        if request.bytes.len() > limit || request.max_output_bytes > limit {
            return Err(CapabilityError::SizeLimit);
        }
        let envelope = kortex_abi::Envelope {
            v: 1,
            request_id: request.request_id.clone(),
            kind: kortex_abi::RequestKind::Init,
            payload: Default::default(),
        };
        if envelope.encode().is_err() {
            return Err(CapabilityError::InvalidRequest);
        }
        let original_scope = request.resource.scope(cap)?;
        let mut deadline =
            self.policy
                .permit_expiry(context, cap, &request.resource, gesture, time());
        let decision = self
            .policy
            .consume(context, cap, &request.resource, gesture, time())?;
        let mut deadline = deadline.take().ok_or(CapabilityError::Denied)?;
        let mut redirects = 0;
        loop {
            let backend = match &request.resource {
                Resource::File { handle } => Some(
                    self.handles
                        .resolve(context, handle, cap, time())?
                        .to_string(),
                ),
                _ => None,
            };
            let addresses = if let Resource::Network { url } = &request.resource {
                let parsed = network_url(url)?;
                let addresses = match parsed.host().ok_or(CapabilityError::NetworkDenied)? {
                    url::Host::Ipv4(ip) => vec![IpAddr::V4(ip)],
                    url::Host::Ipv6(ip) => vec![IpAddr::V6(ip)],
                    url::Host::Domain(host) => self.host.resolve(host).await?,
                };
                if addresses.is_empty() || addresses.iter().any(|ip| !public_ip(*ip)) {
                    return Err(CapabilityError::NetworkDenied);
                }
                addresses
            } else {
                Vec::new()
            };
            if time() >= deadline {
                return Err(CapabilityError::Denied);
            }
            let authorized = AuthorizedCall {
                context,
                request: &request,
                addresses,
                backend,
                expires_ms: deadline,
            };
            let response = self.host.execute(&authorized).await?;
            if time() >= deadline {
                return Err(CapabilityError::Denied);
            }
            match response {
                CapabilityResponse::Bytes(bytes) => {
                    if bytes.len() > request.max_output_bytes {
                        return Err(CapabilityError::SizeLimit);
                    }
                    return Ok(CapabilityResponse::Bytes(bytes));
                }
                CapabilityResponse::Redirect(location) => {
                    if cap != Capability::NetworkFetch {
                        return Err(CapabilityError::NetworkDenied);
                    }
                    redirects += 1;
                    if redirects > 5 {
                        return Err(CapabilityError::RedirectLimit);
                    }
                    let Resource::Network { url } = &request.resource else {
                        return Err(CapabilityError::NetworkDenied);
                    };
                    let target = network_url(url)?
                        .join(&location)
                        .map_err(|_| CapabilityError::NetworkDenied)?;
                    let target = Resource::Network {
                        url: target.to_string(),
                    };
                    let scope = target.scope(cap)?;
                    if scope != original_scope {
                        match self.policy.authorize(context, cap, &target, None, time()) {
                            Decision::AllowOnce | Decision::AllowSession => {
                                let target_expiry = self
                                    .policy
                                    .permit_expiry(context, cap, &target, None, time())
                                    .ok_or(CapabilityError::NetworkDenied)?;
                                self.policy.consume(context, cap, &target, None, time())?;
                                deadline = deadline.min(target_expiry);
                            }
                            _ => return Err(CapabilityError::NetworkDenied),
                        }
                    } else if decision == Decision::Deny || decision == Decision::Ask {
                        return Err(CapabilityError::NetworkDenied);
                    }
                    request.resource = target;
                }
            }
        }
    }
}
