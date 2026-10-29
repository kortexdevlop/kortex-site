use base64::{Engine, engine::general_purpose::STANDARD};
use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
pub fn encode(value: &Value) -> Result<Value, &'static str> {
    let object = value.as_object().ok_or("INVALID_INPUT")?;
    let text = value["text"]
        .as_str()
        .filter(|s| s.len() <= 32768)
        .ok_or("INVALID_INPUT")?;
    if object.len() != 1 {
        return Err("INVALID_INPUT");
    }
    Ok(json!({"base64":STANDARD.encode(text.as_bytes())}))
}
#[derive(Default)]
pub struct Encoder {
    running: bool,
    text: String,
    output: String,
    error: Option<&'static str>,
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
impl Encoder {
    fn apply(&mut self, input: &Value) -> Result<Value, &'static str> {
        match encode(input) {
            Ok(out) => {
                self.text = input["text"].as_str().unwrap().into();
                self.output = out["base64"].as_str().unwrap().into();
                self.error = None;
                Ok(out)
            }
            Err(e) => {
                self.error = Some(e);
                Err(e)
            }
        }
    }
    fn render(&self) -> Value {
        node(
            "column",
            "base64",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "title",
                    json!({"text":"Encodeur Base64","level":1}),
                    vec![],
                ),
                node(
                    "textarea",
                    "source",
                    json!({"label":"Texte UTF-8","value":self.text}),
                    vec![],
                ),
                node(
                    "textarea",
                    "result",
                    json!({"label":"Résultat Base64","value":self.output,"disabled":true}),
                    vec![],
                ),
                node(
                    "alert",
                    "status",
                    json!({"text":self.error.unwrap_or("Traitement local · 32 Kio maximum"),"severity":if self.error.is_some(){"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
impl Tool for Encoder {
    fn handle(&mut self, r: Request) -> Reply {
        let id = r.request_id;
        let p = Value::Object(r.payload);
        let result = match r.kind {
            RequestKind::Init => {
                *self = Self {
                    running: true,
                    ..Default::default()
                };
                Ok(json!({}))
            }
            RequestKind::Shutdown => {
                *self = Self::default();
                Ok(json!({}))
            }
            _ if !self.running => Err("NOT_INITIALIZED"),
            RequestKind::Render => Ok(self.render()),
            RequestKind::ActionInvoke if p["action_id"] == "encode" => self.apply(&p["input"]),
            RequestKind::UiEvent if p["node_key"] == "source" && p["event_type"] == "input" => {
                let _ = self.apply(&json!({"text":p["value"]}));
                Ok(json!({}))
            }
            _ => Err("UNSUPPORTED_REQUEST"),
        };
        match result {
            Ok(v) => success(id, v.as_object().unwrap().clone()),
            Err(e) => failure(id, e),
        }
    }
}
kortex_sdk::export_tool!(Encoder);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encodes_real_utf8_and_empty_input() {
        assert_eq!(encode(&json!({"text":"abc"})).unwrap()["base64"], "YWJj");
        assert_eq!(
            encode(&json!({"text":"Été"})).unwrap()["base64"],
            "w4l0w6k="
        );
        assert_eq!(encode(&json!({"text":""})).unwrap()["base64"], "");
    }
    #[test]
    fn rejects_extra_fields_wrong_types_and_byte_overflow() {
        assert!(encode(&json!({"text":"x","upload":true})).is_err());
        assert!(encode(&json!({"text":null})).is_err());
        assert!(encode(&json!({"text":"é".repeat(16385)})).is_err());
        assert_eq!(
            encode(&json!({"text":"x".repeat(32768)})).unwrap()["base64"]
                .as_str()
                .unwrap()
                .len(),
            43692
        );
    }
}
