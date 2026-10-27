//! SHA-256 and detached Ed25519 proof over the exact B.2 publication text.
pub mod keys;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::LazyLock;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidDescriptor,
    ArchiveLimit,
    HashMismatch,
    UntrustedKey,
    SignatureInvalid,
    InvalidTrustStore,
    Rollback,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub schema_version: u8,
    pub tool_id: String,
    pub version: String,
    pub abi_version: String,
    pub sha256: String,
    pub signing_key_id: String,
    pub signature: String,
}
static SCHEMA: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    jsonschema::validator_for(
        &serde_json::from_str(include_str!("../../../shared/schemas/artifact.schema.json"))
            .expect("artifact schema"),
    )
    .expect("compiled artifact schema")
});
impl Descriptor {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 1048576 {
            return Err(Error::InvalidDescriptor);
        }
        // JSON Schema numbers compare mathematically (1, 1.0 and 1e0 are the same).
        let mut value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| Error::InvalidDescriptor)?;
        if !SCHEMA.is_valid(&value) {
            return Err(Error::InvalidDescriptor);
        }
        value["schema_version"] = serde_json::json!(1);
        let d: Self = serde_json::from_value(value).map_err(|_| Error::InvalidDescriptor)?;
        d.validate()?;
        Ok(d)
    }
    pub fn validate(&self) -> Result<()> {
        if !SCHEMA.is_valid(&serde_json::to_value(self).map_err(|_| Error::InvalidDescriptor)?) {
            return Err(Error::InvalidDescriptor);
        }
        decode::<64>(&self.signature).map_err(|_| Error::InvalidDescriptor)?;
        Ok(())
    }
    pub fn signing_text(&self) -> Result<Vec<u8>> {
        self.validate()?;
        Ok(format!(
            "kortex-artifact-v1\n{}\n{}\n{}\n{}\n",
            self.tool_id, self.version, self.abi_version, self.sha256
        )
        .into_bytes())
    }
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn verify_archive_hash(bytes: &[u8], proof: &Descriptor) -> Result<()> {
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 {
        return Err(Error::ArchiveLimit);
    }
    proof.validate()?;
    if sha256(bytes) != proof.sha256 {
        return Err(Error::HashMismatch);
    }
    Ok(())
}
/// Verify hash before selecting a key or checking its signature. Does not unzip or execute.
pub fn verify_descriptor(
    bytes: &[u8],
    proof: &Descriptor,
    trust: &keys::TrustStore,
    now: u64,
) -> Result<()> {
    verify_archive_hash(bytes, proof)?;
    let key = trust.key(&proof.signing_key_id, now)?;
    verify_signature(&key, &proof.signing_text()?, &proof.signature)
}
/// The caller injects a seed (CLI: CI secret only). It is never included in the proof.
pub fn sign_descriptor(
    bytes: &[u8],
    tool_id: &str,
    version: &str,
    abi_version: &str,
    key_id: &str,
    seed: &[u8; 32],
) -> Result<Descriptor> {
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 {
        return Err(Error::ArchiveLimit);
    }
    let mut proof = Descriptor {
        schema_version: 1,
        tool_id: tool_id.into(),
        version: version.into(),
        abi_version: abi_version.into(),
        sha256: sha256(bytes),
        signing_key_id: key_id.into(),
        signature: STANDARD.encode([0; 64]),
    };
    let text = proof.signing_text()?;
    proof.signature = STANDARD.encode(SigningKey::from_bytes(seed).sign(&text).to_bytes());
    Ok(proof)
}
pub(crate) fn decode<const N: usize>(value: &str) -> Result<[u8; N]> {
    if value.len() != N.div_ceil(3) * 4 {
        return Err(Error::SignatureInvalid);
    }
    let b = STANDARD
        .decode(value)
        .map_err(|_| Error::SignatureInvalid)?;
    if STANDARD.encode(&b) != value {
        return Err(Error::SignatureInvalid);
    }
    b.try_into().map_err(|_| Error::SignatureInvalid)
}
pub(crate) fn verify_signature(public_key: &[u8; 32], text: &[u8], signature: &str) -> Result<()> {
    VerifyingKey::from_bytes(public_key)
        .map_err(|_| Error::SignatureInvalid)?
        .verify_strict(text, &Signature::from_bytes(&decode::<64>(signature)?))
        .map_err(|_| Error::SignatureInvalid)
}
