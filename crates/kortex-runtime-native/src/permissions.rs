//! Trusted native host. The guest can enqueue requests, never grants or gestures.
use crate::{files::NativeFiles, network::NativeNetwork, storage::NativeStorage};
use base64::Engine;
use kortex_capabilities::{
    handles::Access,
    policy::{Context, Duration},
    storage::{Owner, ToolStorage},
    *,
};
use kortex_sdk::CapabilityRequest;
use serde_json::{Map, Value, json};
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
pub async fn blocking<T: Send + 'static>(
    work: impl FnOnce(Arc<AtomicBool>) -> T + Send + 'static,
) -> Result<T, CapabilityError> {
    struct Cancel(Arc<AtomicBool>);
    impl Drop for Cancel {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let cancel = Cancel(Arc::new(AtomicBool::new(false)));
    let flag = cancel.0.clone();
    let (tx, rx) = futures_channel::oneshot::channel();
    std::thread::Builder::new()
        .name("kortex-native-io".into())
        .spawn(move || {
            let _ = tx.send(work(flag));
        })
        .map_err(|_| CapabilityError::HostFailure)?;
    rx.await.map_err(|_| CapabilityError::HostFailure)
}
#[derive(Default)]
struct Inbox {
    generation: u64,
    active: bool,
    queue: VecDeque<CapabilityRequest>,
    ids: std::collections::BTreeSet<String>,
}
#[derive(Clone, Default)]
pub struct CapabilityInbox(Arc<Mutex<Inbox>>);
impl CapabilityInbox {
    pub(crate) fn begin(&self) -> u64 {
        let mut s = self.0.lock().unwrap_or_else(|e| e.into_inner());
        s.generation = s.generation.wrapping_add(1);
        s.active = true;
        s.queue.clear();
        s.ids.clear();
        s.generation
    }
    pub(crate) fn close(&self, generation: u64) {
        if let Ok(mut s) = self.0.lock()
            && s.generation == generation
        {
            s.active = false;
            s.queue.clear();
            s.ids.clear();
        }
    }
    pub(crate) fn enqueue(&self, generation: u64, bytes: &[u8], declared: &[String]) -> i32 {
        if bytes.len() > kortex_abi::MAX_MESSAGE_BYTES {
            return -1;
        }
        let Ok(r) = serde_json::from_slice::<CapabilityRequest>(bytes) else {
            return -1;
        };
        let envelope = kortex_abi::Envelope {
            v: 1,
            request_id: r.request_id.clone(),
            kind: kortex_abi::RequestKind::Init,
            payload: Map::new(),
        };
        if envelope.encode().is_err()
            || !declared.contains(&r.capability)
            || Capability::try_from(r.capability.as_str()).is_err()
        {
            return -1;
        }
        let Ok(mut s) = self.0.lock() else { return -1 };
        if !s.active
            || s.generation != generation
            || s.ids.len() >= 128
            || s.ids.contains(&r.request_id)
        {
            return -1;
        }
        s.ids.insert(r.request_id.clone());
        s.queue.push_back(r);
        0
    }
    pub fn pop(&self) -> Option<CapabilityRequest> {
        self.0.lock().ok()?.queue.pop_front()
    }
    pub fn complete(&self, id: &str) {
        if let Ok(mut s) = self.0.lock() {
            s.ids.remove(id);
        }
    }
}
pub struct NativeHost {
    pub files: NativeFiles,
    storage: Arc<Mutex<NativeStorage>>,
    network: NativeNetwork,
    epoch: Arc<std::sync::atomic::AtomicU64>,
}
impl NativeHost {
    pub fn new(storage: NativeStorage, network: NativeNetwork) -> Self {
        Self {
            files: NativeFiles::default(),
            storage: Arc::new(Mutex::new(storage)),
            network,
            epoch: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    pub fn app_data(network: NativeNetwork) -> Result<Self, CapabilityError> {
        Ok(Self::new(
            NativeStorage::app_data().map_err(|_| CapabilityError::HostFailure)?,
            network,
        ))
    }
    pub fn revoke(&self, context: &Context) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        self.files.revoke(context);
    }
}
impl CapabilityHost for NativeHost {
    async fn resolve(&mut self, host: &str) -> Result<Vec<std::net::IpAddr>, CapabilityError> {
        let host = host.to_string();
        blocking(move |_| NativeNetwork::resolve(&host)).await?
    }
    async fn execute(
        &mut self,
        call: &AuthorizedCall<'_>,
    ) -> Result<CapabilityResponse, CapabilityError> {
        let owner = call.context().clone();
        let cap = call.request().capability;
        let resource = call.request().resource.clone();
        let payload: Value = serde_json::from_slice(&call.request().bytes)
            .map_err(|_| CapabilityError::InvalidRequest)?;
        let backend = call.file_backend().map(str::to_string);
        let addresses = call.addresses().to_vec();
        let expires = call.expires_ms();
        let storage = self.storage.clone();
        let files = self.files.clone();
        let network = self.network.clone();
        let epoch = self.epoch.clone();
        let version = epoch.load(Ordering::SeqCst);
        blocking(move |cancel|{
            let check=||if cancel.load(Ordering::SeqCst)||epoch.load(Ordering::SeqCst)!=version||now_ms()>=expires{Err(CapabilityError::Denied)}else{Ok(())};check()?;
            let result=match (cap,resource){
                (Capability::FilePick,Resource::FileDialog)=>{let path=rfd::FileDialog::new().pick_file();check()?;files.select(&owner,path.as_deref(),now_ms())?},
                (Capability::FileSave,Resource::FileDialog)=>{let bytes=base64::engine::general_purpose::STANDARD.decode(payload["data_base64"].as_str().ok_or(CapabilityError::InvalidRequest)?).map_err(|_|CapabilityError::InvalidRequest)?;let name=payload["name"].as_str().unwrap_or("download.bin");if name.len()>256||name.contains(['/', '\\', ':'])||name.chars().any(char::is_control){return Err(CapabilityError::InvalidRequest)}let path=rfd::FileDialog::new().set_file_name(name).save_file();check()?;NativeFiles::save(path.as_deref(),&bytes)?},
                (Capability::FileRead,Resource::File{..})=>files.read(&owner,backend.as_deref().ok_or(CapabilityError::InvalidHandle)?,payload.get("offset").map(|v|v.as_u64().ok_or(CapabilityError::InvalidRequest)).transpose()?.unwrap_or(0),payload.get("length").map(|v|v.as_u64().ok_or(CapabilityError::InvalidRequest)).transpose()?.unwrap_or(262144).try_into().map_err(|_|CapabilityError::SizeLimit)?,now_ms())?,
                (Capability::StorageRead|Capability::StorageWrite,Resource::Storage{key})=>{if payload["secret"]==true{return Err(CapabilityError::Denied)}let mut backend=storage.lock().map_err(|_|CapabilityError::HostFailure)?;check()?;backend.cancellation(cancel.clone());let mut service=ToolStorage::new(&mut *backend,Owner{profile:"local".into(),tool_id:owner.tool_id.clone()},1).map_err(storage_error)?;
                    if cap==Capability::StorageRead{json!({"value":service.get(&key).map_err(storage_error)?.map(|v|serde_json::from_slice::<Value>(&v)).transpose().map_err(|_|CapabilityError::HostFailure)?})}else{if payload["operation"]=="delete"{service.delete(&key).map_err(storage_error)?}else{service.set(&key,serde_json::to_vec(&payload["value"]).map_err(|_|CapabilityError::InvalidRequest)?).map_err(storage_error)?}json!({"written":true})}},
                (Capability::NetworkFetch,Resource::Network{url})=>return network.fetch(&url,&addresses,std::time::Duration::from_millis(expires.saturating_sub(now_ms()))),
                (Capability::ClipboardWrite,Resource::Clipboard)=>{let text=payload["text"].as_str().ok_or(CapabilityError::InvalidRequest)?;if text.len()>1048000{return Err(CapabilityError::SizeLimit)}check()?;arboard::Clipboard::new().and_then(|mut c|c.set_text(text)).map_err(|_|CapabilityError::HostFailure)?;json!({"written":true})},
                _=>return Err(CapabilityError::Unsupported)
            };check()?;Ok(CapabilityResponse::Bytes(serde_json::to_vec(&result).map_err(|_|CapabilityError::HostFailure)?))
        }).await?
    }
}
fn storage_error(e: storage::StorageError) -> CapabilityError {
    match e {
        storage::StorageError::Quota => CapabilityError::SizeLimit,
        storage::StorageError::Permission => CapabilityError::Denied,
        _ => CapabilityError::HostFailure,
    }
}
pub fn resource(r: &CapabilityRequest) -> Result<Resource, CapabilityError> {
    let p = &r.payload;
    let res = match r.capability.as_str() {
        "file.pick" | "file.save" => Resource::FileDialog,
        "file.read" => Resource::File {
            handle: p["handle"]
                .as_str()
                .ok_or(CapabilityError::InvalidHandle)?
                .into(),
        },
        "storage.read" | "storage.write" => Resource::Storage {
            key: p["key"]
                .as_str()
                .ok_or(CapabilityError::InvalidRequest)?
                .into(),
        },
        "network.fetch" => {
            let u = NativeNetwork::url(p["url"].as_str().ok_or(CapabilityError::NetworkDenied)?)?;
            Resource::Network { url: u.to_string() }
        }
        "clipboard.write" => Resource::Clipboard,
        _ => return Err(CapabilityError::Denied),
    };
    res.scope(Capability::try_from(r.capability.as_str())?)?;
    Ok(res)
}
pub fn host_error(code: &str) -> kortex_abi::HostError {
    kortex_abi::HostError {
        code: code.into(),
        message: code.into(),
        retryable: false,
    }
}
pub struct NativeServices {
    pub gateway: Gateway<NativeHost>,
    files: NativeFiles,
    epoch: Arc<std::sync::atomic::AtomicU64>,
}
impl Drop for NativeServices {
    fn drop(&mut self) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
    }
}
impl NativeServices {
    pub fn new(host: NativeHost) -> Self {
        Self {
            files: host.files.clone(),
            epoch: host.epoch.clone(),
            gateway: Gateway::new(host),
        }
    }
    pub fn revoke(&mut self, owner: &Context) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        self.gateway.revoke_instance(owner);
        self.files.revoke(owner);
    }
    /// Called only by the host's actual consent buttons, never with guest-selected identity.
    pub async fn fulfill(
        &mut self,
        owner: &Context,
        request: &CapabilityRequest,
        allow: bool,
        duration: Duration,
    ) -> Result<Map<String, Value>, kortex_abi::HostError> {
        self.dispatch_decision(owner, request, Some((allow, duration)), false)
            .await
    }
    pub async fn execute_existing(
        &mut self,
        owner: &Context,
        request: &CapabilityRequest,
    ) -> Result<Map<String, Value>, kortex_abi::HostError> {
        self.dispatch_decision(owner, request, None, false).await
    }
    /// Trusted host only: caller must have a fresh actual UI approval for this exact operation.
    /// Consumes the existing scoped grant; does not create a second grant.
    pub async fn execute_existing_with_gesture(&mut self,owner:&Context,request:&CapabilityRequest)->Result<Map<String,Value>,kortex_abi::HostError>{
        self.dispatch_decision(owner,request,None,true).await
    }
    async fn dispatch_decision(
        &mut self,
        owner: &Context,
        request: &CapabilityRequest,
        decision: Option<(bool, Duration)>,
        fresh_ui_gesture: bool,
    ) -> Result<Map<String, Value>, kortex_abi::HostError> {
        let outcome = async {
            let cap = Capability::try_from(request.capability.as_str())?;
            let resource = resource(request)?;
            let gesture = match decision {
                Some((true, duration)) => {
                    self.gateway.policy.grant(
                        owner,
                        cap,
                        &resource,
                        duration,
                        now_ms() + 300000,
                    )?;
                    if cap.needs_gesture() {
                        Some(self.gateway.policy.record_gesture(owner, now_ms())?)
                    } else {
                        None
                    }
                }
                Some((false, _)) => {
                    self.gateway.policy.deny(owner, cap, &resource)?;
                    None
                }
                None if fresh_ui_gesture => Some(self.gateway.policy.record_gesture(owner,now_ms())?),
                None => None,
            };
            let response = self
                .gateway
                .execute(
                    owner,
                    CapabilityCall {
                        request_id: request.request_id.clone(),
                        capability: cap,
                        resource,
                        bytes: serde_json::to_vec(&request.payload)
                            .map_err(|_| CapabilityError::InvalidRequest)?,
                        max_output_bytes: 1048576,
                    },
                    gesture.as_ref(),
                    &now_ms,
                )
                .await?;
            let CapabilityResponse::Bytes(bytes) = response else {
                return Err(CapabilityError::NetworkDenied);
            };
            let mut value: Map<String, Value> =
                serde_json::from_slice(&bytes).map_err(|_| CapabilityError::HostFailure)?;
            if cap == Capability::FilePick && value.get("cancelled") != Some(&Value::Bool(true)) {
                let backend = value
                    .remove("backend")
                    .and_then(|v| v.as_str().map(str::to_string))
                    .ok_or(CapabilityError::InvalidHandle)?;
                let handle = self.gateway.handles.issue(
                    owner.clone(),
                    Access::Read,
                    backend,
                    now_ms(),
                    300000,
                )?;
                value.insert("handle".into(), Value::String(handle));
            }
            Ok(value)
        }
        .await;
        match outcome {
            Ok(v) if v.get("cancelled") == Some(&Value::Bool(true)) => Err(host_error("CANCELLED")),
            Ok(v) => Ok(v),
            Err(e) => Err(host_error(match e {
                CapabilityError::Denied | CapabilityError::NeedsConsent => "PERMISSION",
                CapabilityError::SizeLimit => "QUOTA",
                CapabilityError::InvalidHandle => "INVALID_HANDLE",
                CapabilityError::NetworkDenied | CapabilityError::RedirectLimit => "NETWORK_DENIED",
                CapabilityError::InvalidRequest => "INVALID_INPUT",
                CapabilityError::Unsupported => "UNSUPPORTED",
                _ => "UNAVAILABLE",
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inbox_is_bounded_correlated_and_generation_scoped() {
        let inbox = CapabilityInbox::default();
        let old = inbox.begin();
        let generation = inbox.begin();
        inbox.close(old);
        let declared = vec!["storage.read".to_string()];
        let message = |i| {
            serde_json::to_vec(&serde_json::json!({"request_id":format!("00000000-0000-4000-8000-{i:012x}"),"capability":"storage.read","payload":{"key":"prefs"}})).unwrap()
        };
        assert_eq!(inbox.enqueue(old, &message(0), &declared), -1);
        assert_eq!(inbox.enqueue(generation, &message(0), &[]), -1);
        for i in 0..128 {
            assert_eq!(inbox.enqueue(generation, &message(i), &declared), 0)
        }
        let first = inbox.pop().unwrap();
        assert_eq!(inbox.enqueue(generation, &message(0), &declared), -1);
        assert_eq!(inbox.enqueue(generation, &message(128), &declared), -1);
        inbox.complete(&first.request_id);
        assert_eq!(inbox.enqueue(generation, &message(128), &declared), 0);
        inbox.close(generation);
        assert!(inbox.pop().is_none());
    }
    #[test]
    fn cancelled_io_job_observes_cancellation() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut job = Box::pin(blocking(move |cancel| {
            std::thread::sleep(std::time::Duration::from_millis(20));
            let _ = tx.send(cancel.load(Ordering::SeqCst));
        }));
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        assert!(std::future::Future::poll(job.as_mut(), &mut cx).is_pending());
        drop(job);
        assert!(rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap());
    }
}
