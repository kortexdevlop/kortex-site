#![allow(async_fn_in_trait)]
pub mod files;
pub mod limits;
pub mod network;
pub mod permissions;
pub mod storage;
mod wasmtime;
use kortex_core::{ArtifactDescriptor, RuntimeError, VerifiedTool};
use serde_json::Value;
use std::collections::BTreeMap;
pub use wasmtime::{HostedNativeEngine, NativeGuestEngine, NativeInstance};
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
    let files = kortex_pack::archive::read(&bytes).map_err(|_| RuntimeError::IntegrityError)?;
    kortex_pack::validate::files(&files).map_err(|_| RuntimeError::UnsupportedAbi)?;
    let (m, c, _) = package_parts(&bytes)?;
    VerifiedTool::verify(&m, &c, bytes, proof, keys)
}
pub(crate) fn package_parts(bytes: &[u8]) -> Result<(Value, Value, Vec<u8>), RuntimeError> {
    let mut files = kortex_pack::archive::read(bytes).map_err(|_| RuntimeError::IntegrityError)?;
    let m = files
        .remove("manifest.json")
        .ok_or(RuntimeError::IntegrityError)?;
    let c = files
        .remove("contract.json")
        .ok_or(RuntimeError::IntegrityError)?;
    if m.len() > 1048576 || c.len() > 1048576 {
        return Err(RuntimeError::IntegrityError);
    }
    Ok((
        serde_json::from_slice(&m).map_err(|_| RuntimeError::IntegrityError)?,
        serde_json::from_slice(&c).map_err(|_| RuntimeError::IntegrityError)?,
        files
            .remove("module.wasm")
            .ok_or(RuntimeError::IntegrityError)?,
    ))
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostTrust {
    pub root: kortex_verify::keys::RootKey,
    pub document: kortex_verify::keys::SignedKeys,
    pub minimum_version: u64,
}
pub fn verify_package_with_trust(
    bytes: Vec<u8>,
    proof: &kortex_verify::Descriptor,
    bundle: &HostTrust,
    now: u64,
) -> Result<VerifiedTool, RuntimeError> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    kortex_verify::verify_archive_hash(&bytes, proof).map_err(|_| RuntimeError::IntegrityError)?;
    let trust = kortex_verify::keys::TrustStore::from_signed(
        &bundle.document,
        &bundle.root,
        bundle.minimum_version,
        now,
    )
    .map_err(|_| RuntimeError::UntrustedKey)?;
    kortex_verify::verify_descriptor(&bytes, proof, &trust, now)
        .map_err(|_| RuntimeError::SignatureInvalid)?;
    let key = bundle
        .document
        .keys
        .iter()
        .find(|k| k.id == proof.signing_key_id)
        .ok_or(RuntimeError::UntrustedKey)?;
    let key: [u8; 32] = STANDARD
        .decode(&key.public_key_base64)
        .map_err(|_| RuntimeError::UntrustedKey)?
        .try_into()
        .map_err(|_| RuntimeError::UntrustedKey)?;
    let descriptor = ArtifactDescriptor {
        schema_version: proof.schema_version,
        tool_id: proof.tool_id.clone(),
        version: proof.version.clone(),
        abi_version: proof.abi_version.clone(),
        sha256: proof.sha256.clone(),
        signing_key_id: proof.signing_key_id.clone(),
        signature: STANDARD
            .decode(&proof.signature)
            .map_err(|_| RuntimeError::SignatureInvalid)?
            .try_into()
            .map_err(|_| RuntimeError::SignatureInvalid)?,
    };
    verify_package(
        bytes,
        &descriptor,
        &BTreeMap::from([(proof.signing_key_id.clone(), key)]),
    )
}
