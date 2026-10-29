use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
pub fn convert(action: &str, input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|o| o.len() != 1) {
        return Err("INVALID_INPUT");
    }
    let text = input["text"].as_str().ok_or("INVALID_INPUT")?;
    if text.len() > 32768 {
        return Err("TEXT_TOO_LARGE");
    }
    let result = match action {
        "upper" => text.to_uppercase(),
        "lower" => text.to_lowercase(),
        "title" => {
            let mut initial = true;
            let mut out = String::new();
            for c in text.to_lowercase().chars() {
                if c.is_whitespace() || c == '-' {
                    initial = true;
                    out.push(c);
                } else if initial && c.is_alphabetic() {
                    out.extend(c.to_uppercase());
                    initial = false;
                } else {
                    out.push(c);
                }
            }
            out
        }
        _ => return Err("UNKNOWN_ACTION"),
    };
    Ok(json!({"text":result}))
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
#[derive(Default)]
pub struct CaseConverter {
    text: String,
    result: String,
    mode: String,
    error: Option<&'static str>,
    running: bool,
}
impl CaseConverter {
    fn apply(&mut self, action: &str, input: &Value) -> Result<Value, &'static str> {
        match convert(action, input) {
            Ok(out) => {
                self.text = input["text"].as_str().unwrap().into();
                self.mode = action.into();
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
            "case-converter",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "case-heading",
                    json!({"text":"Convertisseur de casse","level":1}),
                    vec![],
                ),
                node(
                    "textarea",
                    "text",
                    json!({"label":"Texte à convertir","value":self.text}),
                    vec![],
                ),
                node(
                    "row",
                    "actions",
                    json!({"gap":8}),
                    [
                        ("upper", "Majuscules"),
                        ("lower", "Minuscules"),
                        ("title", "Initiales"),
                    ]
                    .into_iter()
                    .map(|(key, label)| node("button", key, json!({"text":label}), vec![]))
                    .collect(),
                ),
                node(
                    "textarea",
                    "converted",
                    json!({"label":"Texte converti","value":self.result,"disabled":true}),
                    vec![],
                ),
                node(
                    "alert",
                    "status",
                    json!({"text":self.error.unwrap_or("Traitement local · 32 Kio UTF-8 maximum"),"severity":if self.error.is_some(){"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
impl Tool for CaseConverter {
    fn handle(&mut self, request: Request) -> Reply {
        let id = request.request_id;
        let p = Value::Object(request.payload);
        let out = match request.kind {
            RequestKind::Init => {
                *self = Self {
                    running: true,
                    mode: "lower".into(),
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
                    let mode = self.mode.clone();
                    let _ = self.apply(&mode, &json!({"text":p["value"]}));
                    Ok(json!({}))
                } else if p["event_type"] == "click"
                    && p["value"].is_null()
                    && ["upper", "lower", "title"].contains(&key)
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
kortex_sdk::export_tool!(CaseConverter);
