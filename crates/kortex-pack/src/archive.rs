//! Strict ZIP subset: stored/deflate, no descriptors, extras, comments, ZIP64 or links.
use crate::{Files, Issue, MAX_ARCHIVE, MAX_FILE, MAX_FILES, Result};
use std::{
    collections::BTreeSet,
    io::{Cursor, Read, Write},
};
fn bad() -> Issue {
    Issue::new("INVALID_ARCHIVE", "")
}
fn u16(b: &[u8], p: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(p..p + 2).ok_or_else(bad)?.try_into().unwrap(),
    ))
}
fn u32(b: &[u8], p: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(p..p + 4).ok_or_else(bad)?.try_into().unwrap(),
    ))
}
pub fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 256
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
        && name.split('/').all(|p| {
            let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with('.')
                && !matches!(
                    stem.as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "COM4"
                        | "COM5"
                        | "COM6"
                        | "COM7"
                        | "COM8"
                        | "COM9"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                        | "LPT4"
                        | "LPT5"
                        | "LPT6"
                        | "LPT7"
                        | "LPT8"
                        | "LPT9"
                )
        })
}
/// Inspect all headers and limits before asking the ZIP library to decompress any entry.
pub fn read(bytes: &[u8]) -> Result<Files> {
    if bytes.len() < 22 || bytes.len() > MAX_ARCHIVE {
        return Err(Issue::new("ARCHIVE_LIMIT", ""));
    }
    let end = bytes.len() - 22;
    if u32(bytes, end)? != 0x06054b50
        || u16(bytes, end + 4)? != 0
        || u16(bytes, end + 6)? != 0
        || u16(bytes, end + 20)? != 0
    {
        return Err(bad());
    }
    let count = u16(bytes, end + 10)? as usize;
    let offset = u32(bytes, end + 16)? as usize;
    if count == 0
        || count > MAX_FILES
        || u16(bytes, end + 8)? as usize != count
        || offset.checked_add(u32(bytes, end + 12)? as usize) != Some(end)
    {
        return Err(bad());
    }
    let mut at = offset;
    let mut total = 0usize;
    let mut names = BTreeSet::new();
    let mut entries = Vec::new();
    let mut ranges = Vec::new();
    for _ in 0..count {
        if at + 46 > end || u32(bytes, at)? != 0x02014b50 {
            return Err(bad());
        }
        let flags = u16(bytes, at + 8)?;
        let method = u16(bytes, at + 10)?;
        let crc = u32(bytes, at + 16)?;
        let compressed = u32(bytes, at + 20)? as usize;
        let len = u32(bytes, at + 24)? as usize;
        let nl = u16(bytes, at + 28)? as usize;
        let local = u32(bytes, at + 42)? as usize;
        let attrs = u32(bytes, at + 38)?;
        let mode = attrs >> 16;
        if u16(bytes, at + 6)? > 20
            || flags & !0x800 != 0
            || ![0, 8].contains(&method)
            || u16(bytes, at + 30)? != 0
            || u16(bytes, at + 32)? != 0
            || u16(bytes, at + 34)? != 0
            || attrs & 0x18 != 0
            || ![0, 0x8000].contains(&(mode & 0xf000))
            || at + 46 + nl > end
        {
            return Err(bad());
        }
        let name = std::str::from_utf8(&bytes[at + 46..at + 46 + nl])
            .map_err(|_| bad())?
            .to_string();
        if !safe_name(&name) || !names.insert(name.to_ascii_lowercase()) {
            return Err(Issue::new("INVALID_PATH", &name));
        }
        total = total.checked_add(len).ok_or_else(bad)?;
        if len > MAX_FILE || total > MAX_ARCHIVE || len > compressed.max(1).saturating_mul(20) {
            return Err(Issue::new("ARCHIVE_LIMIT", &name));
        }
        if local.checked_add(30).is_none_or(|n| n > offset)
            || u32(bytes, local)? != 0x04034b50
            || u16(bytes, local + 4)? != u16(bytes, at + 6)?
            || u16(bytes, local + 6)? != flags
            || u16(bytes, local + 8)? != method
            || u32(bytes, local + 14)? != crc
            || u32(bytes, local + 18)? as usize != compressed
            || u32(bytes, local + 22)? as usize != len
            || u16(bytes, local + 26)? as usize != nl
            || u16(bytes, local + 28)? != 0
        {
            return Err(bad());
        }
        let start = local + 30 + nl;
        let stop = start.checked_add(compressed).ok_or_else(bad)?;
        if stop > offset || bytes.get(local + 30..start) != Some(name.as_bytes()) {
            return Err(bad());
        }
        ranges.push((local, stop));
        entries.push((name, len, crc));
        at += 46 + nl;
    }
    if at != end {
        return Err(bad());
    }
    ranges.sort_unstable();
    let mut next = 0;
    for (start, stop) in ranges {
        if start != next {
            return Err(bad());
        }
        next = stop;
    }
    if next != offset {
        return Err(bad());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| bad())?;
    let mut files = Files::new();
    for (index, (name, len, crc)) in entries.into_iter().enumerate() {
        let mut f = archive.by_index(index).map_err(|_| bad())?;
        if f.name().map_err(|_| bad())?.as_ref() != name || f.size() != len as u64 {
            return Err(bad());
        }
        let mut data = Vec::with_capacity(len);
        f.by_ref()
            .take(len as u64 + 1)
            .read_to_end(&mut data)
            .map_err(|_| Issue::new("INVALID_CRC", &name))?;
        if data.len() != len || crc32fast::hash(&data) != crc {
            return Err(Issue::new("INVALID_CRC", &name));
        }
        files.insert(name, data);
    }
    Ok(files)
}
pub fn write(files: &Files) -> Result<Vec<u8>> {
    if files.is_empty() || files.len() > MAX_FILES {
        return Err(Issue::new("ARCHIVE_LIMIT", ""));
    }
    let mut total = 0;
    let mut names = BTreeSet::new();
    for (name, data) in files {
        if !safe_name(name) || !names.insert(name.to_ascii_lowercase()) {
            return Err(Issue::new("INVALID_PATH", name));
        }
        total += data.len();
        if data.len() > MAX_FILE || total > MAX_ARCHIVE {
            return Err(Issue::new("ARCHIVE_LIMIT", name));
        }
    }
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .system(zip::System::Unix)
        .compression_method(zip::CompressionMethod::Stored)
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644);
    for (name, data) in files {
        zip.start_file(name, options).map_err(|_| bad())?;
        zip.write_all(data).map_err(|_| bad())?;
    }
    let bytes = zip.finish().map_err(|_| bad())?.into_inner();
    if bytes.len() > MAX_ARCHIVE {
        return Err(Issue::new("ARCHIVE_LIMIT", ""));
    }
    Ok(bytes)
}
