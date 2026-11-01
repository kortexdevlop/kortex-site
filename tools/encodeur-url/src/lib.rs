use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
const INPUT_LIMIT: usize = 32768;
const HEX: &[u8; 16] = b"0123456789ABCDEF";
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 15) as usize] as char);
        }
    }
    out
}
fn decode(text: &str) -> Result<String, &'static str> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let digits = bytes.get(i + 1..i + 3).ok_or("INVALID_PERCENT")?;
            let hex = std::str::from_utf8(digits).map_err(|_| "INVALID_PERCENT")?;
            if !hex.bytes().all(|d| d.is_ascii_hexdigit()) {
                return Err("INVALID_PERCENT");
            }
            out.push(u8::from_str_radix(hex, 16).unwrap());
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "INVALID_UTF8")
}
/// Headless action: the UI calls the very same code. `+` is never read as a space.
pub fn convert(action: &str, input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|o| o.len() != 1) {
        return Err("INVALID_INPUT");
    }
    let text = input["text"].as_str().ok_or("INVALID_INPUT")?;
    if text.len() > INPUT_LIMIT {
        return Err("TEXT_TOO_LARGE");
    }
    let out = match action {
        "encode" => encode(text),
        "decode" => decode(text)?,
        _ => return Err("UNKNOWN_ACTION"),
    };
    Ok(json!({"text":out}))
}
fn explain(code: &str) -> &'static str {
    match code {
        "INVALID_PERCENT" => "Séquence « % » incomplète ou non hexadécimale.",
        "INVALID_UTF8" => "Les octets décodés ne forment pas un texte UTF-8 valide.",
        "TEXT_TOO_LARGE" => "Le texte dépasse 32 Kio.",
        _ => "Opération impossible.",
    }
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
#[derive(Default)]
pub struct UrlEncoder {
    text: String,
    result: String,
    error: Option<&'static str>,
    running: bool,
}
impl UrlEncoder {
    fn apply(&mut self, action: &str, input: &Value) -> Result<Value, &'static str> {
        match convert(action, input) {
            Ok(out) => {
                self.result = out["text"].as_str().unwrap().into();
                self.error = None;
                Ok(out)
            }
            Err(e) => {
                self.result.clear();
                self.error = Some(e);
                Err(e)
            }
        }
    }
    fn render(&self) -> Value {
        node(
            "column",
            "url-encoder",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "url-heading",
                    json!({"text":"Encodeur URL","level":1}),
                    vec![],
                ),
                node(
                    "textarea",
                    "text",
                    json!({"label":"Texte à traiter","value":self.text}),
                    vec![],
                ),
                node(
                    "row",
                    "actions",
                    json!({"gap":8}),
                    [("encode", "Encoder"), ("decode", "Décoder")]
                        .into_iter()
                        .map(|(key, label)| node("button", key, json!({"text":label}), vec![]))
                        .collect(),
                ),
                node(
                    "textarea",
                    "converted",
                    json!({"label":"Résultat","value":self.result,"disabled":true}),
                    vec![],
                ),
                node(
                    "alert",
                    "status",
                    json!({"text":self.error.map_or("Traitement local · 32 Kio UTF-8 maximum · « + » reste un plus",explain),"severity":if self.error.is_some(){"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
impl Tool for UrlEncoder {
    fn handle(&mut self, request: Request) -> Reply {
        let id = request.request_id;
        let p = Value::Object(request.payload);
        let out = match request.kind {
            RequestKind::Init => {
                *self = Self {
                    running: true,
                    ..Self::default()
                };
                Ok(json!({}))
            }
            RequestKind::Shutdown => {
                *self = Self::default();
                Ok(json!({}))
            }
            _ if !self.running => Err("NOT_INITIALIZED"),
            RequestKind::Render => Ok(self.render()),
            RequestKind::ActionInvoke => {
                self.apply(p["action_id"].as_str().unwrap_or(""), &p["input"])
            }
            RequestKind::UiEvent => {
                let key = p["node_key"].as_str().unwrap_or("");
                if key == "text" && p["event_type"] == "input" {
                    self.text = p["value"].as_str().unwrap_or("").into();
                    self.result.clear();
                    self.error = None;
                    Ok(json!({}))
                } else if p["event_type"] == "click"
                    && p["value"].is_null()
                    && ["encode", "decode"].contains(&key)
                {
                    let text = self.text.clone();
                    let _ = self.apply(key, &json!({"text":text}));
                    Ok(json!({}))
                } else {
                    Err("INVALID_INPUT")
                }
            }
            _ => Err("UNSUPPORTED_REQUEST"),
        };
        match out {
            Ok(v) => success(id, v.as_object().unwrap().clone()),
            Err(e) => failure(id, e),
        }
    }
}
kortex_sdk::export_tool!(UrlEncoder);
