use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
pub fn count(value: &Value) -> Result<Value, &'static str> {
    let object = value.as_object().ok_or("INVALID_INPUT")?;
    let text = value["text"]
        .as_str()
        .filter(|s| s.len() <= 32768)
        .ok_or("INVALID_INPUT")?;
    if object.len() != 1 {
        return Err("INVALID_INPUT");
    }
    Ok(json!({
        "words": text.split_whitespace().count(),
        "characters": text.chars().count(),
        "characters_no_spaces": text.chars().filter(|c| !c.is_whitespace()).count(),
        "lines": text.lines().count(),
    }))
}
#[derive(Default)]
pub struct Counter {
    running: bool,
    text: String,
    output: String,
    error: Option<&'static str>,
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
impl Counter {
    fn apply(&mut self, input: &Value) -> Result<Value, &'static str> {
        match count(input) {
            Ok(out) => {
                self.text = input["text"].as_str().unwrap().into();
                self.output = format!(
                    "{} mots · {} caractères · {} sans espaces · {} lignes",
                    out["words"], out["characters"], out["characters_no_spaces"], out["lines"]
                );
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
            "compteur",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "title",
                    json!({"text":"Compteur de mots","level":1}),
                    vec![],
                ),
                node(
                    "textarea",
                    "source",
                    json!({"label":"Texte","value":self.text}),
                    vec![],
                ),
                node(
                    "textarea",
                    "result",
                    json!({"label":"Résultat","value":self.output,"disabled":true}),
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
impl Tool for Counter {
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
            RequestKind::ActionInvoke if p["action_id"] == "count" => self.apply(&p["input"]),
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
kortex_sdk::export_tool!(Counter);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_unicode_words_characters_and_lines() {
        let out = count(&json!({"text":"Été  à
la mer"}))
        .unwrap();
        assert_eq!(
            out,
            json!({"words":4,"characters":14,"characters_no_spaces":10,"lines":2})
        );
    }
    #[test]
    fn empty_is_zero_and_oversized_or_extra_fields_are_refused() {
        assert_eq!(count(&json!({"text":""})).unwrap()["words"], 0);
        assert!(count(&json!({"text":"a".repeat(32769)})).is_err());
        assert!(count(&json!({"text":"a","x":1})).is_err());
    }
}
