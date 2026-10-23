use kortex_abi::{Envelope, HostError, MAX_MESSAGE_BYTES, MAX_QUEUED_EVENTS};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
pub type PendingId = String;
pub type CapabilityOutcome = Result<Map<String, Value>, HostError>;
type Callback = Box<dyn FnOnce(CapabilityOutcome) + Send>;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequest {
    pub request_id: PendingId,
    pub capability: String,
    pub payload: Map<String, Value>,
}
struct Pending {
    callback: Option<Callback>,
}
pub struct Host {
    pending: BTreeMap<PendingId, Pending>,
    transport: fn(&[u8]) -> i32,
}
fn error(code: &str) -> HostError {
    HostError {
        code: code.into(),
        message: code.into(),
        retryable: false,
    }
}
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "kortex_v1")]
unsafe extern "C" {
    fn host_enqueue(ptr: i32, len: i32) -> i32;
}
fn transport(bytes: &[u8]) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { host_enqueue(bytes.as_ptr() as i32, bytes.len() as i32) }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = bytes;
        -1
    }
}
impl Default for Host {
    fn default() -> Self {
        Self::new(transport)
    }
}
impl Host {
    pub fn new(transport: fn(&[u8]) -> i32) -> Self {
        Self {
            pending: BTreeMap::new(),
            transport,
        }
    }
    pub fn enqueue(&mut self, request: CapabilityRequest) -> Result<PendingId, HostError> {
        // UUID validation uses the frozen envelope codec, not a second permissive parser.
        let envelope = Envelope {
            v: 1,
            request_id: request.request_id.clone(),
            kind: kortex_abi::RequestKind::Init,
            payload: Map::new(),
        };
        if envelope.encode().is_err() {
            return Err(error("INVALID_REQUEST_ID"));
        }
        if ![
            "storage.read",
            "storage.write",
            "file.pick",
            "file.read",
            "file.save",
            "network.fetch",
            "clipboard.write",
        ]
        .contains(&request.capability.as_str())
        {
            return Err(error("UNKNOWN_CAPABILITY"));
        }
        if self.pending.len() >= MAX_QUEUED_EVENTS {
            return Err(error("QUEUE_FULL"));
        }
        if self.pending.contains_key(&request.request_id) {
            return Err(error("DUPLICATE_REQUEST"));
        }
        let bytes = serde_json::to_vec(&request).map_err(|_| error("INVALID_REQUEST"))?;
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(error("MESSAGE_TOO_LARGE"));
        }
        // Install correlation before enqueue; the ABI forbids synchronous reentrant result delivery.
        let id = request.request_id;
        self.pending.insert(id.clone(), Pending { callback: None });
        let status = (self.transport)(&bytes);
        if status != 0 {
            self.pending.remove(&id);
            return Err(error(if status < 0 {
                "CAPABILITY_DENIED"
            } else {
                "INVALID_HOST_STATUS"
            }));
        }
        Ok(id)
    }
    pub fn on_result(
        &mut self,
        id: &str,
        callback: impl FnOnce(CapabilityOutcome) + Send + 'static,
    ) -> Result<(), HostError> {
        let pending = self
            .pending
            .get_mut(id)
            .ok_or_else(|| error("UNKNOWN_CAPABILITY_RESULT"))?;
        if pending.callback.is_some() {
            return Err(error("CALLBACK_ALREADY_REGISTERED"));
        }
        pending.callback = Some(Box::new(callback));
        Ok(())
    }
    pub fn resolve(&mut self, request: &Envelope) -> Result<(), HostError> {
        if request.kind != kortex_abi::RequestKind::CapabilityResult || request.encode().is_err() {
            return Err(error("INVALID_CAPABILITY_RESULT"));
        }
        let id = request.payload["capability_request_id"]
            .as_str()
            .ok_or_else(|| error("INVALID_CAPABILITY_RESULT"))?;
        let pending = self
            .pending
            .remove(id)
            .ok_or_else(|| error("UNKNOWN_CAPABILITY_RESULT"))?;
        let outcome = if request.payload["ok"] == true {
            Ok(request
                .payload
                .get("data")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default())
        } else {
            Err(serde_json::from_value(request.payload["error"].clone())
                .map_err(|_| error("INVALID_CAPABILITY_RESULT"))?)
        };
        if let Some(callback) = pending.callback {
            callback(outcome)
        }
        Ok(())
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}
