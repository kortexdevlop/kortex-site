use crate::{MAX_MESSAGE_BYTES, PROTOCOL_VERSION};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestKind {
    #[serde(rename = "init")]
    Init,
    #[serde(rename = "render")]
    Render,
    #[serde(rename = "ui.event")]
    UiEvent,
    #[serde(rename = "action.invoke")]
    ActionInvoke,
    #[serde(rename = "capability.result")]
    CapabilityResult,
    #[serde(rename = "state.restore")]
    StateRestore,
    #[serde(rename = "shutdown")]
    Shutdown,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub v: u8,
    pub request_id: String,
    pub kind: RequestKind,
    pub payload: Map<String, Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub v: u8,
    pub request_id: String,
    pub ok: bool,
    #[serde(
        default,
        deserialize_with = "non_null_optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub data: Option<Map<String, Value>>,
    #[serde(
        default,
        deserialize_with = "non_null_optional",
        skip_serializing_if = "Option::is_none"
    )]
    pub error: Option<HostError>,
}
fn non_null_optional<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapabilityResult {
    capability_request_id: String,
    ok: bool,
    #[serde(default, deserialize_with = "non_null_optional")]
    data: Option<Map<String, Value>>,
    #[serde(default, deserialize_with = "non_null_optional")]
    error: Option<HostError>,
}
fn check_payload(kind: &RequestKind, payload: &Map<String, Value>) -> Result<(), CodecError> {
    if *kind == RequestKind::CapabilityResult {
        let result: CapabilityResult = serde_json::from_value(Value::Object(payload.clone()))
            .map_err(CodecError::InvalidJson)?;
        Reply {
            v: PROTOCOL_VERSION,
            request_id: result.capability_request_id,
            ok: result.ok,
            data: result.data,
            error: result.error,
        }
        .validate()?;
    }
    Ok(())
}
#[derive(Debug)]
pub enum CodecError {
    Empty,
    TooLarge,
    InvalidJson(serde_json::Error),
    InvalidEnvelope(&'static str),
}
fn check_bytes(bytes: &[u8]) -> Result<(), CodecError> {
    if bytes.is_empty() {
        return Err(CodecError::Empty);
    }
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(CodecError::TooLarge);
    }
    Ok(())
}
fn check_header(v: u8, id: &str) -> Result<(), CodecError> {
    if v != PROTOCOL_VERSION {
        return Err(CodecError::InvalidEnvelope("unsupported protocol version"));
    }
    let b = id.as_bytes();
    if b.len() != 36
        || !b.iter().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                *c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
    {
        return Err(CodecError::InvalidEnvelope(
            "request_id must be an opaque UUID",
        ));
    }
    Ok(())
}
impl Envelope {
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        check_bytes(bytes)?;
        let result: Self = serde_json::from_slice(bytes).map_err(CodecError::InvalidJson)?;
        check_header(result.v, &result.request_id)?;
        check_payload(&result.kind, &result.payload)?;
        Ok(result)
    }
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        check_header(self.v, &self.request_id)?;
        check_payload(&self.kind, &self.payload)?;
        let bytes = serde_json::to_vec(self).map_err(CodecError::InvalidJson)?;
        check_bytes(&bytes)?;
        Ok(bytes)
    }
}
impl Reply {
    fn validate(&self) -> Result<(), CodecError> {
        check_header(self.v, &self.request_id)?;
        if self.ok && self.error.is_some()
            || !self.ok && (self.error.is_none() || self.data.is_some())
        {
            return Err(CodecError::InvalidEnvelope("inconsistent reply outcome"));
        }
        if let Some(error) = &self.error {
            if error.code.is_empty()
                || error.message.is_empty()
                || error.code.chars().count() > 100
                || error.message.chars().count() > 2000
            {
                return Err(CodecError::InvalidEnvelope("empty error fields"));
            }
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        check_bytes(bytes)?;
        let result: Self = serde_json::from_slice(bytes).map_err(CodecError::InvalidJson)?;
        result.validate()?;
        Ok(result)
    }
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(CodecError::InvalidJson)?;
        check_bytes(&bytes)?;
        Ok(bytes)
    }
}
