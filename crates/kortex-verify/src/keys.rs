use crate::{Error, Result, decode, verify_signature};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const MAX_SAFE: u64 = 9_007_199_254_740_991;
pub fn key_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustKey {
    pub id: String,
    pub public_key_base64: String,
    pub not_before: u64,
    pub expires: u64,
    pub revoked: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootKey {
    pub id: String,
    pub public_key_base64: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedKeys {
    pub schema_version: u8,
    pub version: u64,
    pub issued_at: u64,
    pub expires: u64,
    pub root_key_id: String,
    pub keys: Vec<TrustKey>,
    pub signature: String,
}
impl SignedKeys {
    pub fn signing_text(&self) -> Result<Vec<u8>> {
        if self.schema_version != 1
            || self.version == 0
            || self.version > MAX_SAFE
            || self.issued_at >= self.expires
            || self.expires > MAX_SAFE
            || !key_id(&self.root_key_id)
            || self.keys.len() > 256
        {
            return Err(Error::InvalidTrustStore);
        }
        let mut keys = self.keys.clone();
        keys.sort_by(|a, b| a.id.cmp(&b.id));
        let mut previous = None;
        let mut text = format!(
            "kortex-keys-v1\n{}\n{}\n{}\n{}\n",
            self.version, self.issued_at, self.expires, self.root_key_id
        );
        for k in &keys {
            if !key_id(&k.id)
                || previous == Some(k.id.as_str())
                || k.not_before >= k.expires
                || k.expires > MAX_SAFE
            {
                return Err(Error::InvalidTrustStore);
            }
            decode::<32>(&k.public_key_base64).map_err(|_| Error::InvalidTrustStore)?;
            text.push_str(&format!(
                "{}|{}|{}|{}|{}\n",
                k.id, k.public_key_base64, k.not_before, k.expires, k.revoked
            ));
            previous = Some(&k.id);
        }
        Ok(text.into_bytes())
    }
}
/// Immutable trust snapshot. Root pinning and minimum version are supplied by the host.
pub struct TrustStore {
    keys: BTreeMap<String, TrustKey>,
    expires: u64,
    pub version: u64,
}
impl TrustStore {
    pub fn from_signed(
        document: &SignedKeys,
        root: &RootKey,
        minimum_version: u64,
        now: u64,
    ) -> Result<Self> {
        let text = document.signing_text()?;
        if document.version < minimum_version {
            return Err(Error::Rollback);
        }
        if !key_id(&root.id)
            || root.id != document.root_key_id
            || now < document.issued_at
            || now >= document.expires
        {
            return Err(Error::UntrustedKey);
        }
        verify_signature(
            &decode::<32>(&root.public_key_base64)?,
            &text,
            &document.signature,
        )?;
        Ok(Self {
            keys: document
                .keys
                .iter()
                .map(|k| (k.id.clone(), k.clone()))
                .collect(),
            expires: document.expires,
            version: document.version,
        })
    }
    pub(crate) fn key(&self, id: &str, now: u64) -> Result<[u8; 32]> {
        let k = self.keys.get(id).ok_or(Error::UntrustedKey)?;
        if now >= self.expires || k.revoked || now < k.not_before || now >= k.expires {
            return Err(Error::UntrustedKey);
        }
        decode::<32>(&k.public_key_base64).map_err(|_| Error::UntrustedKey)
    }
}
