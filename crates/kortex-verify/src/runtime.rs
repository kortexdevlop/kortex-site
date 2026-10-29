use crate::{
    Error, Result, decode,
    keys::{RootKey, SignedKeys, TrustStore},
    verify_signature,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Web {
    pub origin: String,
    pub version_url: String,
    pub bundle_sha256: String,
    pub script_integrity: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Desktop {
    pub version: String,
    pub platform: String,
    pub url: String,
    pub sha256: String,
    pub code_signed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub runtime_version: String,
    pub abi_versions: Vec<String>,
    pub protocol_versions: Vec<u8>,
    pub ui_schema_versions: Vec<String>,
    pub web: Web,
    pub desktop: Option<Desktop>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Channels {
    pub stable: Option<Release>,
    pub staging: Option<Release>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub schema_version: u8,
    pub sequence: u64,
    pub issued_at: u64,
    pub expires: u64,
    pub channels: Channels,
    pub signing_key_id: String,
    pub signature: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub document: Document,
    pub keys: SignedKeys,
}
fn version(v: &str) -> bool {
    let parts: Vec<_> = v.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && (!p.starts_with('0') || *p == "0")
                && p.bytes().all(|b| b.is_ascii_digit())
                && p.parse::<u64>().is_ok()
        })
}
fn digest(v: &str) -> bool {
    v.len() == 64
        && v.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn canonical(v: &Value) -> Result<String> {
    Ok(match v {
        Value::Null | Value::Bool(_) | Value::String(_) => {
            serde_json::to_string(v).map_err(|_| Error::InvalidDescriptor)?
        }
        Value::Number(n) if n.as_u64().is_some_and(|n| n <= 9_007_199_254_740_991) => n.to_string(),
        Value::Array(a) => format!(
            "[{}]",
            a.iter()
                .map(canonical)
                .collect::<Result<Vec<_>>>()?
                .join(",")
        ),
        Value::Object(o) => {
            let mut keys: Vec<_> = o.keys().collect();
            keys.sort();
            format!(
                "{{{}}}",
                keys.into_iter()
                    .map(|k| Ok(format!(
                        "{}:{}",
                        serde_json::to_string(k).map_err(|_| Error::InvalidDescriptor)?,
                        canonical(&o[k])?
                    )))
                    .collect::<Result<Vec<_>>>()?
                    .join(",")
            )
        }
        _ => return Err(Error::InvalidDescriptor),
    })
}
impl Document {
    pub fn signing_text(&self) -> Result<Vec<u8>> {
        if self.schema_version != 1
            || self.sequence == 0
            || self.sequence > 9_007_199_254_740_991
            || self.expires > 9_007_199_254_740_991
            || self.issued_at >= self.expires
            || self.expires - self.issued_at > 30 * 86_400_000
            || !crate::keys::key_id(&self.signing_key_id)
        {
            return Err(Error::InvalidDescriptor);
        }
        for (r, stable) in [
            (&self.channels.stable, true),
            (&self.channels.staging, false),
        ] {
            if let Some(r) = r {
                let u = url::Url::parse(&r.web.origin).map_err(|_| Error::InvalidDescriptor)?;
                if !version(&r.runtime_version)
                    || r.abi_versions != ["1.0"]
                    || r.protocol_versions != [1]
                    || r.ui_schema_versions != ["1.0"]
                    || u.scheme() != "https"
                    || u.origin().ascii_serialization() != r.web.origin
                    || r.web.version_url != format!("{}/version.json", r.web.origin)
                    || !digest(&r.web.bundle_sha256)
                {
                    return Err(Error::InvalidDescriptor);
                }
                decode::<32>(
                    r.web
                        .script_integrity
                        .strip_prefix("sha256-")
                        .ok_or(Error::InvalidDescriptor)?,
                )?;
                if let Some(d) = &r.desktop {
                    let u = url::Url::parse(&d.url).map_err(|_| Error::InvalidDescriptor)?;
                    if !version(&d.version)
                        || d.platform != "windows-x64"
                        || !digest(&d.sha256)
                        || u.scheme() != "https"
                        || u.host_str() != Some("github.com")
                        || !u
                            .path()
                            .starts_with("/kortexdevlop/kortex-site/releases/download/")
                        || !u.username().is_empty()
                        || u.password().is_some()
                        || u.query().is_some()
                        || u.fragment().is_some()
                        || stable && !d.code_signed
                    {
                        return Err(Error::InvalidDescriptor);
                    }
                }
            }
        }
        let mut value = serde_json::to_value(self).map_err(|_| Error::InvalidDescriptor)?;
        value
            .as_object_mut()
            .ok_or(Error::InvalidDescriptor)?
            .remove("signature");
        Ok(format!("kortex-runtime-channels-v1\n{}\n", canonical(&value)?).into_bytes())
    }
}
impl Envelope {
    pub fn verify(&self, root: &RootKey, minimum_sequence: u64, now: u64) -> Result<()> {
        let d = &self.document;
        let text = d.signing_text()?;
        if d.sequence < minimum_sequence {
            return Err(Error::Rollback);
        }
        if now < d.issued_at || now >= d.expires {
            return Err(Error::UntrustedKey);
        }
        let trust = TrustStore::from_signed(&self.keys, root, 1, now)?;
        verify_signature(&trust.key(&d.signing_key_id, now)?, &text, &d.signature)
    }
}
