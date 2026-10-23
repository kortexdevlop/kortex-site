//! Guest SDK. No WASI, filesystem, network or browser access.
//!
//! ```
//! use kortex_sdk::{Tool, Request, Reply};
//! #[derive(Default)]
//! struct Example;
//! impl Tool for Example {
//!     fn handle(&mut self, request: Request) -> Reply {
//!         kortex_sdk::success(request.request_id, Default::default())
//!     }
//! }
//! // At the root of a cdylib guest crate:
//! kortex_sdk::export_tool!(Example);
//! ```
pub mod capability;
pub mod ffi;
pub use capability::{CapabilityRequest, Host, PendingId};
pub use kortex_abi::{Envelope as Request, HostError, Reply, RequestKind};
use serde_json::{Map, Value};
pub const FALLBACK_ID: &str = "00000000-0000-4000-8000-000000000000";

pub trait Tool {
    fn handle(&mut self, request: Request) -> Reply;
    fn host(&mut self) -> Option<&mut Host> {
        None
    }
}
pub fn success(request_id: String, data: Map<String, Value>) -> Reply {
    Reply {
        v: 1,
        request_id,
        ok: true,
        data: Some(data),
        error: None,
    }
}
pub fn failure(request_id: String, code: &str) -> Reply {
    Reply {
        v: 1,
        request_id,
        ok: false,
        data: None,
        error: Some(HostError {
            code: code.into(),
            message: code.into(),
            retryable: false,
        }),
    }
}
pub struct Dispatcher<T: Tool> {
    pub tool: T,
}
impl<T: Tool> Dispatcher<T> {
    pub fn new(tool: T) -> Self {
        Self { tool }
    }
    pub fn dispatch(&mut self, bytes: &[u8]) -> Vec<u8> {
        let request = match Request::decode(bytes) {
            Ok(request) => request,
            Err(_) => return encode(failure(FALLBACK_ID.into(), "INVALID_REQUEST")),
        };
        let id = request.request_id.clone();
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if request.kind == RequestKind::CapabilityResult {
                match self.tool.host() {
                    Some(host) => {
                        if let Err(error) = host.resolve(&request) {
                            return failure(id.clone(), &error.code);
                        }
                    }
                    None => return failure(id.clone(), "UNKNOWN_CAPABILITY_RESULT"),
                }
            }
            let reply = self.tool.handle(request);
            if reply.request_id != id {
                failure(id.clone(), "CORRELATION_ERROR")
            } else {
                reply
            }
        }));
        encode(outcome.unwrap_or_else(|_| failure(id, "TOOL_PANIC")))
    }
}
fn encode(reply: Reply) -> Vec<u8> {
    match reply.encode() {
        Ok(bytes)=>bytes,
        Err(_)=>failure(reply.request_id,"INVALID_REPLY").encode().unwrap_or_else(|_|b"{\"v\":1,\"request_id\":\"00000000-0000-4000-8000-000000000000\",\"ok\":false,\"error\":{\"code\":\"INVALID_REPLY\",\"message\":\"INVALID_REPLY\",\"retryable\":false}}".to_vec()),
    }
}
#[macro_export]
macro_rules! export_tool {
    ($tool:ty) => {
        #[cfg(target_arch = "wasm32")]
        static KORTEX_TOOL: std::sync::OnceLock<std::sync::Mutex<$crate::Dispatcher<$tool>>> =
            std::sync::OnceLock::new();
        #[cfg(target_arch = "wasm32")]
        #[unsafe(no_mangle)]
        pub extern "C" fn kortex_alloc(len: i32) -> i32 {
            $crate::ffi::allocate(len)
        }
        #[cfg(target_arch = "wasm32")]
        #[unsafe(no_mangle)]
        pub extern "C" fn kortex_dealloc(ptr: i32, len: i32) {
            $crate::ffi::deallocate(ptr, len);
        }
        #[cfg(target_arch = "wasm32")]
        #[unsafe(no_mangle)]
        pub extern "C" fn kortex_dispatch(ptr: i32, len: i32) -> i64 {
            let reply = match $crate::ffi::request_bytes(ptr, len) {
                Ok(bytes) => {
                    let mutex = KORTEX_TOOL.get_or_init(|| {
                        std::sync::Mutex::new($crate::Dispatcher::new(<$tool>::default()))
                    });
                    match mutex.try_lock() {
                        Ok(mut dispatcher) => dispatcher.dispatch(&bytes),
                        Err(_) => $crate::failure($crate::FALLBACK_ID.into(), "REENTRANT_DISPATCH")
                            .encode()
                            .unwrap_or_default(),
                    }
                }
                Err(_) => $crate::failure($crate::FALLBACK_ID.into(), "INVALID_BUFFER")
                    .encode()
                    .unwrap_or_default(),
            };
            $crate::ffi::response_bytes(reply)
        }
    };
}
