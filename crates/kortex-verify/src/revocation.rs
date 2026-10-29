use crate::{
    Descriptor, Error, Result, decode,
    keys::{RootKey, key_id},
    verify_signature,
};
use serde::{Deserialize, Serialize};
pub const TTL: u64 = 86400000;
const MAX: u64 = 9_007_199_254_740_991;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub tool_id: String,
    pub version: String,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub schema_version: u8,
    pub version: u64,
    pub issued_at: u64,
    pub valid_until: u64,
    pub root_key_id: String,
    pub revoked_keys: Vec<String>,
    pub revoked_artifacts: Vec<Artifact>,
    pub signature: String,
}
impl Document {
    pub fn signing_text(&self) -> Result<Vec<u8>> {
        let bad = || Error::InvalidTrustStore;
        if self.schema_version != 1
            || self.version == 0
            || self.version > MAX
            || self.issued_at >= self.valid_until
            || self.valid_until > MAX
            || self.valid_until - self.issued_at > TTL
            || !key_id(&self.root_key_id)
            || self.revoked_keys.len() > 256
            || self.revoked_artifacts.len() > 4096
        {
            return Err(bad());
        }
        let mut keys = self.revoked_keys.clone();
        keys.sort();
        if keys.windows(2).any(|w| w[0] == w[1]) || keys.iter().any(|k| !key_id(k)) {
            return Err(bad());
        }
        let mut rows = Vec::new();
        for a in &self.revoked_artifacts {
            if a.tool_id.is_empty()
                || a.tool_id.len() > 64
                || !a
                    .tool_id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || a.tool_id.starts_with('-')
                || a.tool_id.ends_with('-')
                || a.tool_id.contains("--")
                || !(5..=128).contains(&a.version.len())
                || !a
                    .version
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".+-".contains(&b))
                || a.sha256.len() != 64
                || !a
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(bad());
            }
            rows.push(format!("{}|{}|{}", a.tool_id, a.version, a.sha256))
        }
        rows.sort();
        if rows.windows(2).any(|w| w[0] == w[1]) {
            return Err(bad());
        }
        Ok(format!(
            "kortex-revocations-v1\n{}\n{}\n{}\n{}\n{}{}",
            self.version,
            self.issued_at,
            self.valid_until,
            self.root_key_id,
            keys.iter()
                .map(|k| format!("key|{k}\n"))
                .collect::<String>(),
            rows.iter()
                .map(|a| format!("artifact|{a}\n"))
                .collect::<String>()
        )
        .into_bytes())
    }
    pub fn verify(&self, root: &RootKey) -> Result<()> {
        if self.root_key_id != root.id {
            return Err(Error::UntrustedKey);
        }
        verify_signature(
            &decode::<32>(&root.public_key_base64)?,
            &self.signing_text()?,
            &self.signature,
        )
    }
    pub fn revoked(&self, p: &Descriptor) -> bool {
        self.revoked_keys.contains(&p.signing_key_id)
            || self
                .revoked_artifacts
                .iter()
                .any(|a| a.tool_id == p.tool_id && a.version == p.version && a.sha256 == p.sha256)
    }
    pub fn fresh(&self, now: u64) -> bool {
        self.issued_at <= now && now < self.valid_until
    }
    pub fn successor(&self, old: &Self) -> bool {
        self.version >= old.version
            && (self.version != old.version || self == old)
            && old
                .revoked_keys
                .iter()
                .all(|k| self.revoked_keys.contains(k))
            && old
                .revoked_artifacts
                .iter()
                .all(|a| self.revoked_artifacts.contains(a))
    }
}
