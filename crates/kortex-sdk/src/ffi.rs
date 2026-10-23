use kortex_abi::{MAX_MEMORY_BYTES, MAX_MESSAGE_BYTES, pointer};
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    InvalidLength,
    UnknownAllocation,
    LengthMismatch,
    MemoryBudget,
}
/// Tracks exact ownership. Native builds use virtual u32 offsets, never truncated native pointers.
pub struct BufferArena {
    buffers: BTreeMap<u32, Box<[u8]>>,
    used: usize,
    #[cfg(not(target_arch = "wasm32"))]
    next: u32,
}
impl Default for BufferArena {
    fn default() -> Self {
        Self {
            buffers: BTreeMap::new(),
            used: 0,
            #[cfg(not(target_arch = "wasm32"))]
            next: 8,
        }
    }
}
impl BufferArena {
    pub fn allocate(&mut self, len: u32) -> Result<u32, BufferError> {
        if len == 0 || len as usize > MAX_MESSAGE_BYTES {
            return Err(BufferError::InvalidLength);
        }
        if self.used + len as usize > MAX_MEMORY_BYTES {
            return Err(BufferError::MemoryBudget);
        }
        let mut data = Vec::new();
        data.try_reserve_exact(len as usize)
            .map_err(|_| BufferError::MemoryBudget)?;
        data.resize(len as usize, 0);
        let data = data.into_boxed_slice();
        #[cfg(target_arch = "wasm32")]
        let ptr = data.as_ptr() as u32;
        #[cfg(not(target_arch = "wasm32"))]
        let ptr = {
            let ptr = self.next;
            self.next = self
                .next
                .checked_add(len)
                .and_then(|x| x.checked_add(8))
                .ok_or(BufferError::MemoryBudget)?;
            ptr
        };
        self.buffers.insert(ptr, data);
        self.used += len as usize;
        Ok(ptr)
    }
    pub fn read(&self, ptr: u32, len: u32) -> Result<Vec<u8>, BufferError> {
        let data = self
            .buffers
            .get(&ptr)
            .ok_or(BufferError::UnknownAllocation)?;
        if data.len() != len as usize {
            return Err(BufferError::LengthMismatch);
        }
        #[cfg(target_arch = "wasm32")]
        let memory_len = core::arch::wasm32::memory_size::<0>() * 65536;
        #[cfg(not(target_arch = "wasm32"))]
        let memory_len = MAX_MEMORY_BYTES;
        pointer::checked_range(ptr, len, memory_len).map_err(|_| BufferError::InvalidLength)?;
        Ok(data.to_vec())
    }
    pub fn write(&mut self, ptr: u32, bytes: &[u8]) -> Result<(), BufferError> {
        let data = self
            .buffers
            .get_mut(&ptr)
            .ok_or(BufferError::UnknownAllocation)?;
        if data.len() != bytes.len() {
            return Err(BufferError::LengthMismatch);
        }
        data.copy_from_slice(bytes);
        Ok(())
    }
    pub fn deallocate(&mut self, ptr: u32, len: u32) -> Result<(), BufferError> {
        let data = self
            .buffers
            .get(&ptr)
            .ok_or(BufferError::UnknownAllocation)?;
        if data.len() != len as usize {
            return Err(BufferError::LengthMismatch);
        }
        self.used -= data.len();
        self.buffers.remove(&ptr);
        Ok(())
    }
    pub fn live_bytes(&self) -> usize {
        self.used
    }
}
static ARENA: OnceLock<Mutex<BufferArena>> = OnceLock::new();
fn arena() -> std::sync::MutexGuard<'static, BufferArena> {
    ARENA
        .get_or_init(|| Mutex::new(BufferArena::default()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}
pub fn allocate(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    arena()
        .allocate(len as u32)
        .map(|ptr| ptr as i32)
        .unwrap_or(0)
}
pub fn deallocate(ptr: i32, len: i32) {
    let _ = arena().deallocate(ptr as u32, len as u32);
}
pub fn request_bytes(ptr: i32, len: i32) -> Result<Vec<u8>, BufferError> {
    arena().read(ptr as u32, len as u32)
}
pub fn response_bytes(bytes: Vec<u8>) -> i64 {
    let mut arena = arena();
    let len = bytes.len() as u32;
    match arena
        .allocate(len)
        .and_then(|ptr| arena.write(ptr, &bytes).map(|_| ptr))
    {
        Ok(ptr) => pointer::pack(ptr, len),
        Err(_) => {
            // No valid reply can be produced without memory. End this guest instance, never return a forged pointer.
            #[cfg(target_arch = "wasm32")]
            core::arch::wasm32::unreachable();
            #[cfg(not(target_arch = "wasm32"))]
            {
                0
            }
        }
    }
}
