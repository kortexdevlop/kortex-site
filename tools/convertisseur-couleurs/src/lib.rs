use kortex_sdk::{Reply, Request, RequestKind, Tool, failure, success};
use serde_json::{Value, json};
const INPUT_LIMIT: usize = 64;
fn args<'a>(text: &'a str, name: &str) -> Option<Vec<&'a str>> {
    let inner = text
        .strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')?;
    let parts: Vec<&str> = inner
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .collect();
    (parts.len() == 3).then_some(parts)
}
fn percent(part: &str) -> Option<f64> {
    let v: f64 = part.strip_suffix('%')?.parse().ok()?;
    (0.0..=100.0).contains(&v).then_some(v / 100.0)
}
fn parse(text: &str) -> Result<[u8; 3], &'static str> {
    let t = text.trim().to_ascii_lowercase();
    if let Some(hex) = t.strip_prefix('#') {
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("INVALID_COLOR");
        }
        let digit = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).unwrap();
        let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap();
        return match hex.len() {
            3 => Ok([digit(0) * 17, digit(1) * 17, digit(2) * 17]),
            6 => Ok([pair(0), pair(2), pair(4)]),
            _ => Err("INVALID_COLOR"),
        };
    }
    if let Some(p) = args(&t, "rgb") {
        let channel = |s: &str| s.parse::<u8>().map_err(|_| "INVALID_COLOR");
        return Ok([channel(p[0])?, channel(p[1])?, channel(p[2])?]);
    }
    if let Some(p) = args(&t, "hsl") {
        let h: f64 = p[0].parse().map_err(|_| "INVALID_COLOR")?;
        let (s, l) = (
            percent(p[1]).ok_or("INVALID_COLOR")?,
            percent(p[2]).ok_or("INVALID_COLOR")?,
        );
        if !(0.0..=360.0).contains(&h) {
            return Err("INVALID_COLOR");
        }
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let hp = (h % 360.0) / 60.0;
        let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
        let (r, g, b) = match hp as u8 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = l - c / 2.0;
        let byte = |v: f64| ((v + m) * 255.0).round() as u8;
        return Ok([byte(r), byte(g), byte(b)]);
    }
    Err("INVALID_COLOR")
}
fn to_hsl([r, g, b]: [u8; 3]) -> (u16, u8, u8) {
    let (r, g, b) = (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let (l, d) = ((max + min) / 2.0, max - min);
    if d == 0.0 {
        return (0, 0, (l * 100.0).round() as u8);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } * 60.0;
    (
        (h.round() as u16) % 360,
        (s * 100.0).round() as u8,
        (l * 100.0).round() as u8,
    )
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
    if !["hex", "rgb", "hsl"].contains(&action) {
        return Err("UNKNOWN_ACTION");
    }
    let rgb = parse(text)?;
    let out = match action {
        "hex" => format!("#{:02x}{:02x}{:02x}", rgb[0], rgb[1], rgb[2]),
        "rgb" => format!("rgb({}, {}, {})", rgb[0], rgb[1], rgb[2]),
        _ => {
            let (h, s, l) = to_hsl(rgb);
            format!("hsl({h}, {s}%, {l}%)")
        }
    };
    Ok(json!({"text":out}))
}
fn explain(code: &str) -> &'static str {
    match code {
        "INVALID_COLOR" => {
            "Couleur non reconnue : utilisez #1f4fa3, rgb(31, 79, 163) ou hsl(218, 68%, 38%)."
        }
        "TEXT_TOO_LARGE" => "Le texte dépasse 64 octets.",
        _ => "Opération impossible.",
    }
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
#[derive(Default)]
pub struct ColorConverter {
    text: String,
    result: String,
    error: Option<&'static str>,
    running: bool,
}
impl ColorConverter {
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
            "color-converter",
            json!({"gap":12}),
            vec![
                node(
                    "heading",
                    "color-heading",
                    json!({"text":"Convertisseur de couleurs","level":1}),
                    vec![],
                ),
                node(
                    "input",
                    "text",
                    json!({"label":"Couleur (HEX, RGB ou HSL)","value":self.text}),
                    vec![],
                ),
                node(
                    "row",
                    "actions",
                    json!({"gap":8}),
                    [("hex", "En HEX"), ("rgb", "En RGB"), ("hsl", "En HSL")]
                        .into_iter()
                        .map(|(key, label)| node("button", key, json!({"text":label}), vec![]))
                        .collect(),
                ),
                node(
                    "input",
                    "converted",
                    json!({"label":"Couleur convertie","value":self.result,"disabled":true}),
                    vec![],
                ),
                node(
                    "alert",
                    "status",
                    json!({"text":self.error.map_or("Traitement local · 64 octets maximum",explain),"severity":if self.error.is_some(){"error"}else{"info"}}),
                    vec![],
                ),
            ],
        )
    }
}
impl Tool for ColorConverter {
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
                    && ["hex", "rgb", "hsl"].contains(&key)
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
kortex_sdk::export_tool!(ColorConverter);
