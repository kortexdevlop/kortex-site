use crate::{MAX_MEMORY_BYTES, MAX_MESSAGE_BYTES};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerError {
    Empty,
    MessageTooLarge,
    MemoryTooLarge,
    Overflow,
    OutOfBounds,
}

pub fn pack(ptr: u32, len: u32) -> i64 {
    (((ptr as u64) << 32) | len as u64) as i64
}
pub fn unpack(value: i64) -> (u32, u32) {
    let bits = value as u64;
    ((bits >> 32) as u32, bits as u32)
}

/// Validate a message range before copying bytes from guest linear memory.
/// Pointer zero is allowed; an empty message is not valid UTF-8 JSON.
pub fn checked_range(ptr: u32, len: u32, memory_len: usize) -> Result<Range<usize>, PointerError> {
    if len == 0 {
        return Err(PointerError::Empty);
    }
    if len as usize > MAX_MESSAGE_BYTES {
        return Err(PointerError::MessageTooLarge);
    }
    if memory_len > MAX_MEMORY_BYTES {
        return Err(PointerError::MemoryTooLarge);
    }
    let end = ptr.checked_add(len).ok_or(PointerError::Overflow)? as usize;
    if end > memory_len {
        return Err(PointerError::OutOfBounds);
    }
    Ok(ptr as usize..end)
}
