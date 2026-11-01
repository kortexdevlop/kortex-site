use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
const INPUT_LIMIT: usize = 16384;
const LINE_LIMIT: usize = 400;
/// Headless action: the UI calls the very same code. Lines are compared exactly.
pub fn diff(input: &Value) -> Result<Value, &'static str> {
    if input.as_object().is_none_or(|o| o.len() != 2) {
        return Err("INVALID_INPUT");
    }
    let side = |key: &str| -> Result<Vec<&str>, &'static str> {
        let text = input[key].as_str().ok_or("INVALID_INPUT")?;
        if text.len() > INPUT_LIMIT {
            return Err("TEXT_TOO_LARGE");
        }
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() > LINE_LIMIT {
            return Err("TOO_MANY_LINES");
        }
        Ok(lines)
    };
    let (a, b) = (side("a")?, side("b")?);
    // lcs[i][j] = length of the longest common subsequence of a[i..] and b[j..].
    let mut lcs = vec![vec![0u16; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut added, mut removed) = (0, 0, 0, 0);
    let mut out: Vec<String> = Vec::new();
    while i < a.len() || j < b.len() {
        if i < a.len() && j < b.len() && a[i] == b[j] {
            out.push(format!("  {}", a[i]));
            i += 1;
            j += 1;
        } else if i < a.len() && (j == b.len() || lcs[i + 1][j] >= lcs[i][j + 1]) {
            out.push(format!("- {}", a[i]));
            removed += 1;
            i += 1;
        } else {
            out.push(format!("+ {}", b[j]));
            added += 1;
            j += 1;
        }
    }
    Ok(json!({"text":out.join("\n"),"added":added,"removed":removed}))
}
fn explain(code: &str) -> &'static str {
    match code {
        "TEXT_TOO_LARGE" => "Un des textes dépasse 16 Kio.",
        "TOO_MANY_LINES" => "Un des textes dépasse 400 lignes.",
        _ => "Opération impossible.",
    }
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
#[derive(Default)]
pub struct TextComparer {
    a: String,
    b: String,
    result: Option<Value>,
    error: Option<&'static str>,
    running: bool,
}
impl TextComparer {
    fn apply(&mut self, input: &Value) -> Result<Value, &'static str> {
        match diff(input) {
            Ok(out) => {
                self.result = Some(out.clone());
                self.error = None;
                Ok(out)
            }
            Err(e) => {
                self.result = None;
                self.error = Some(e);
                Err(e)
            }
        }
    }
    fn render(&self) -> Value {
        let status = match (&self.result, self.error) {
            (_, Some(e)) => explain(e).to_string(),
            (Some(r), _) if r["added"] == 0 && r["removed"] == 0 => {
                "Les deux textes sont identiques.".into()
            }
            (Some(r), _) => format!(
                "{} ligne(s) ajoutée(s), {} supprimée(s).",
                r["added"], r["removed"]
            ),
            _ => "Traitement local · 16 Kio et 400 lignes maximum par texte".into(),
        };
        node(
            "column",
            "text-comparer",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "compare-heading",
                    json!({"text":"Comparateur de texte","level":1}),
                    vec![],
                ),
                node(
                    "textarea",
                    "a",
                    json!({"label":"Texte d’origine","value":self.a}),
                    vec![],
                ),
                node(
                    "textarea",
                    "b",
                    json!({"label":"Texte modifié","value":self.b}),
                    vec![],
                ),
                node(
                    "row",
                    "actions",
                    json!({"gap":8}),
                    vec![node(
                        "button",
                        "compare",
                        json!({"text":"Comparer"}),
                        vec![],
                    )],
                ),
                node(
                    "textarea",
                    "converted",
                    json!({"label":"Différences (- supprimé, + ajouté)","value":self.result.as_ref().map_or("",|r|r["text"].as_str().unwrap()),"disabled":true}),
                    vec![],
                ),
                node(
                    "alert",
                    "status",
                    json!({"text":status,"severity":if self.error.is_some(){"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
impl Tool for TextComparer {
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
                if p["action_id"] == "diff" {
                    self.apply(&p["input"])
                } else {
                    Err("UNKNOWN_ACTION")
                }
            }
            RequestKind::UiEvent => {
                let key = p["node_key"].as_str().unwrap_or("");
                if ["a", "b"].contains(&key) && p["event_type"] == "input" {
                    let value = p["value"].as_str().unwrap_or("").to_string();
                    if key == "a" {
                        self.a = value;
                    } else {
                        self.b = value;
                    }
                    self.result = None;
                    self.error = None;
                    Ok(json!({}))
                } else if key == "compare" && p["event_type"] == "click" && p["value"].is_null() {
                    let input = json!({"a":self.a,"b":self.b});
                    let _ = self.apply(&input);
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
kortex_sdk::export_tool!(TextComparer);
