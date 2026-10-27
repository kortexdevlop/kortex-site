#![allow(async_fn_in_trait)]
pub mod limits;
mod wasmtime;
use kortex_core::{ArtifactDescriptor, RuntimeError, VerifiedTool};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
};
pub use wasmtime::{NativeGuestEngine, NativeInstance};
/// Extract into bounded memory only; signatures are verified before any guest compilation.
pub fn verify_package(
    bytes: Vec<u8>,
    proof: &ArtifactDescriptor,
    keys: &BTreeMap<String, [u8; 32]>,
) -> Result<VerifiedTool, RuntimeError> {
    use sha2::{Digest, Sha256};
    if bytes.len() > 25 * 1024 * 1024
        || proof.schema_version != 1
        || format!("{:x}", Sha256::digest(&bytes)) != proof.sha256
    {
        return Err(RuntimeError::IntegrityError);
    }
    let key = keys
        .get(&proof.signing_key_id)
        .ok_or(RuntimeError::UntrustedKey)?;
    let key =
        ed25519_dalek::VerifyingKey::from_bytes(key).map_err(|_| RuntimeError::SignatureInvalid)?;
    let text = format!(
        "kortex-artifact-v1\n{}\n{}\n{}\n{}\n",
        proof.tool_id, proof.version, proof.abi_version, proof.sha256
    );
    key.verify_strict(
        text.as_bytes(),
        &ed25519_dalek::Signature::from_bytes(&proof.signature),
    )
    .map_err(|_| RuntimeError::SignatureInvalid)?;
    let (m, c, _) = package_parts(&bytes)?;
    VerifiedTool::verify(&m, &c, bytes, proof, keys)
}
pub(crate) fn package_parts(bytes: &[u8]) -> Result<(Value, Value, Vec<u8>), RuntimeError> {
    let bad = || RuntimeError::IntegrityError;
    if bytes.len() > 25 * 1024 * 1024 {
        return Err(bad());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|_| bad())?;
    if archive.len() > 256 {
        return Err(bad());
    }
    let mut names = BTreeSet::new();
    let mut files = BTreeMap::new();
    let mut total = 0u64;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).map_err(|_| bad())?;
        let name = f.name().to_string();
        if name.is_empty()
            || name.len() > 256
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
            || name
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == "..")
            || !names.insert(name.to_lowercase())
            || f.unix_mode()
                .is_some_and(|m| m & 0xf000 != 0 && m & 0xf000 != 0x8000)
        {
            return Err(bad());
        }
        total = total.checked_add(f.size()).ok_or_else(bad)?;
        if total > 50 * 1024 * 1024
            || f.size() > 20 * 1024 * 1024
            || f.size() > f.compressed_size().max(1) * 100
        {
            return Err(bad());
        }
        let mut data = Vec::new();
        let size = f.size();
        f.by_ref()
            .take(size + 1)
            .read_to_end(&mut data)
            .map_err(|_| bad())?;
        if data.len() as u64 != size {
            return Err(bad());
        }
        files.insert(name, data);
    }
    let m = files.remove("manifest.json").ok_or_else(bad)?;
    let c = files.remove("contract.json").ok_or_else(bad)?;
    if m.len() > 1048576 || c.len() > 1048576 {
        return Err(bad());
    }
    Ok((
        serde_json::from_slice(&m).map_err(|_| bad())?,
        serde_json::from_slice(&c).map_err(|_| bad())?,
        files.remove("module.wasm").ok_or_else(bad)?,
    ))
}
