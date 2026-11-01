use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
const INPUT_LIMIT: usize = 2048;
/// ASCII transliteration of a lowercase Latin letter; `None` when there is none.
fn fold(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'ç' | 'ć' | 'č' => "c",
        'ď' | 'đ' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => "e",
        'ğ' => "g",
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ı' => "i",
        'ł' => "l",
        'ñ' | 'ń' | 'ň' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' => "o",
        'ř' => "r",
        'ś' | 'š' | 'ş' => "s",
        'ť' => "t",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' => "u",
        'ý' | 'ÿ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        _ => return None,
    })
}
/// Headless action: the UI calls the very same code.
pub fn convert(action: &str, input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|o| o.len() != 1) {
        return Err("INVALID_INPUT");
    }
    let text = input["text"].as_str().ok_or("INVALID_INPUT")?;
    if text.len() > INPUT_LIMIT {
        return Err("TEXT_TOO_LARGE");
    }
    let separator = match action {
        "slug" => '-',
        "snake" => '_',
        _ => return Err("UNKNOWN_ACTION"),
    };
    let mut out = String::new();
    let mut pending = false;
    for c in text.to_lowercase().chars() {
        // Combining marks (e.g. from "e\u{301}" or "İ") attach to the previous letter.
        if ('\u{300}'..='\u{36f}').contains(&c) {
            continue;
        }
        let piece = if c.is_ascii_alphanumeric() {
            Some(c.to_string())
        } else {
            fold(c).map(str::to_string)
        };
        match piece {
            Some(p) => {
                if pending && !out.is_empty() {
                    out.push(separator);
                }
                pending = false;
                out.push_str(&p);
            }
            None => pending = true,
        }
    }
    if out.is_empty() {
        return Err("EMPTY_RESULT");
    }
    Ok(json!({"text":out}))
}
fn explain(code: &str) -> &'static str {
    match code {
        "EMPTY_RESULT" => "Aucune lettre latine ou chiffre à conserver dans ce texte.",
        "TEXT_TOO_LARGE" => "Le texte dépasse 2 Kio.",
        _ => "Opération impossible.",
    }
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
#[derive(Default)]
pub struct SlugGenerator {
    text: String,
    result: String,
    error: Option<&'static str>,
    running: bool,
}
impl SlugGenerator {
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
            "slug-generator",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "slug-heading",
                    json!({"text":"Générateur de slug","level":1}),
                    vec![],
                ),
                node(
                    "textarea",
                    "text",
                    json!({"label":"Titre ou texte","value":self.text}),
                    vec![],
                ),
                node(
                    "row",
                    "actions",
                    json!({"gap":8}),
                    [("slug", "Slug d’URL"), ("snake", "snake_case")]
                        .into_iter()
                        .map(|(key, label)| node("button", key, json!({"text":label}), vec![]))
                        .collect(),
                ),
                node(
                    "input",
                    "converted",
                    json!({"label":"Résultat","value":self.result,"disabled":true}),
                    vec![],
                ),
                node(
                    "alert",
                    "status",
                    json!({"text":self.error.map_or("Traitement local · 2 Kio UTF-8 maximum · alphabet latin",explain),"severity":if self.error.is_some(){"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
impl Tool for SlugGenerator {
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
                    && ["slug", "snake"].contains(&key)
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
kortex_sdk::export_tool!(SlugGenerator);
