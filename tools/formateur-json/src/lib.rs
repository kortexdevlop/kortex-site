use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
const INPUT_LIMIT: usize = 32768;
const OUTPUT_LIMIT: usize = 98304;
/// Headless action: the UI calls the very same code. Object keys come out sorted.
pub fn convert(action: &str, input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|o| o.len() != 1) {
        return Err("INVALID_INPUT");
    }
    let text = input["text"].as_str().ok_or("INVALID_INPUT")?;
    if text.len() > INPUT_LIMIT {
        return Err("TEXT_TOO_LARGE");
    }
    if !["format", "minify"].contains(&action) {
        return Err("UNKNOWN_ACTION");
    }
    let value: Value = serde_json::from_str(text).map_err(|_| "INVALID_JSON")?;
    let out = if action == "format" {
        serde_json::to_string_pretty(&value)
    } else {
        serde_json::to_string(&value)
    }
    .map_err(|_| "INVALID_JSON")?;
    if out.len() > OUTPUT_LIMIT {
        return Err("OUTPUT_TOO_LARGE");
    }
    Ok(json!({"text":out}))
}
fn explain(code: &str) -> &'static str {
    match code {
        "INVALID_JSON" => "Ce texte n’est pas un JSON valide.",
        "TEXT_TOO_LARGE" => "Le texte dépasse 32 Kio.",
        "OUTPUT_TOO_LARGE" => "Le résultat dépasse la taille autorisée.",
        _ => "Opération impossible.",
    }
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
#[derive(Default)]
pub struct JsonFormatter {
    text: String,
    result: String,
    error: Option<&'static str>,
    running: bool,
}
impl JsonFormatter {
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
            "json-formatter",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "json-heading",
                    json!({"text":"Formateur JSON","level":1}),
                    vec![],
                ),
                node(
                    "textarea",
                    "text",
                    json!({"label":"JSON à traiter","value":self.text}),
                    vec![],
                ),
                node(
                    "row",
                    "actions",
                    json!({"gap":8}),
                    [("format", "Indenter"), ("minify", "Compacter")]
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
                    json!({"text":self.error.map_or("Traitement local · 32 Kio UTF-8 maximum · clés triées",explain),"severity":if self.error.is_some(){"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
impl Tool for JsonFormatter {
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
                    && ["format", "minify"].contains(&key)
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
kortex_sdk::export_tool!(JsonFormatter);
