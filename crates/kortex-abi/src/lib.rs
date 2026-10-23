//! Portable Core WASM ABI definitions; no execution engine or native pointers.
pub mod pointer;
pub mod wire;
pub use wire::{Envelope, HostError, Reply, RequestKind};

pub const ABI_VERSION: &str = "1.0";
pub const PROTOCOL_VERSION: u8 = 1;
pub const IMPORT_MODULE: &str = "kortex_v1";
pub const IMPORT_ENQUEUE: &str = "host_enqueue";
pub const EXPORT_MEMORY: &str = "memory";
pub const EXPORT_ALLOC: &str = "kortex_alloc";
pub const EXPORT_DEALLOC: &str = "kortex_dealloc";
pub const EXPORT_DISPATCH: &str = "kortex_dispatch";
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
pub const MAX_UI_NODES: usize = 2000;
pub const MAX_PACKAGE_BYTES: usize = 25 * 1024 * 1024;
pub const MAX_MODULE_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_MEMORY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_MEMORY_PAGES: u32 = 1024;
pub const MAX_ACTION_MS: u32 = 30_000;
pub const MAX_QUEUED_EVENTS: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnqueueStatus {
    Accepted,
    Rejected(i32),
}
impl TryFrom<i32> for EnqueueStatus {
    type Error = &'static str;
    fn try_from(code: i32) -> Result<Self, Self::Error> {
        match code {
            0 => Ok(Self::Accepted),
            n if n < 0 => Ok(Self::Rejected(n)),
            _ => Err("unknown positive enqueue status"),
        }
    }
}
