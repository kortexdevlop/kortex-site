pub mod build;
pub mod discovery;
pub mod package;
pub type Result<T> = std::result::Result<T, String>;
pub fn read_bounded(path: &std::path::Path, limit: usize) -> Result<Vec<u8>> {
    use std::io::Read;
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if discovery::link(&meta) || !meta.is_file() || meta.len() > limit as u64 {
        return Err("INVALID_FILE".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err("FILE_LIMIT".into());
    }
    Ok(bytes)
}
