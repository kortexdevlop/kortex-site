use serde_json::{Value, json};
const LIMIT: f64 = 9_007_199_254_740_991.0;
#[derive(Default)]
pub struct State {
    a: String,
    b: String,
    target_b: bool,
    result: Option<f64>,
    error: Option<&'static str>,
}
fn number(v: &Value) -> Result<f64, &'static str> {
    v.as_f64()
        .filter(|n| n.is_finite() && n.abs() <= LIMIT)
        .ok_or("INVALID_INPUT")
}
fn node(kind: &str, key: &str, props: Value, children: Vec<Value>) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":children})
}
fn button(key: &str, text: &str) -> Value {
    node("button", key, json!({"text":text}), vec![])
}
impl State {
    /// The sole arithmetic path, also used by all four operation buttons.
    pub fn calculate(&mut self, action: &str, input: &Value) -> Result<Value, &'static str> {
        let outcome = (|| {
            let object = input.as_object().ok_or("INVALID_INPUT")?;
            if object.len() != 2 {
                return Err("INVALID_INPUT");
            }
            let a = number(&input["a"])?;
            let b = number(&input["b"])?;
            let result = match action {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" if b == 0.0 => return Err("DIVIDE_BY_ZERO"),
                "divide" => a / b,
                _ => return Err("UNKNOWN_ACTION"),
            };
            if !result.is_finite() || result.abs() > LIMIT {
                return Err("INVALID_INPUT");
            }
            Ok(if result == 0.0 { 0.0 } else { result })
        })();
        match outcome {
            Ok(result) => {
                self.result = Some(result);
                self.error = None;
                Ok(json!({"result":result}))
            }
            Err(code) => {
                self.result = None;
                self.error = Some(code);
                Err(code)
            }
        }
    }
    pub fn event(&mut self, p: &Value) -> Result<(), &'static str> {
        let key = p["node_key"].as_str().ok_or("INVALID_INPUT")?;
        let kind = p["event_type"].as_str().ok_or("INVALID_INPUT")?;
        if kind == "input" && matches!(key, "a" | "b") {
            let value = p["value"]
                .as_str()
                .filter(|s| s.len() <= 128)
                .ok_or("INVALID_INPUT")?;
            self.target_b = key == "b";
            if self.target_b {
                self.b = value.into()
            } else {
                self.a = value.into()
            }
            self.error = None;
            self.result = None;
            return Ok(());
        }
        if kind != "click" || !p["value"].is_null() {
            return Err("INVALID_INPUT");
        }
        match key {
            "add" | "subtract" | "multiply" | "divide" => {
                let parse = |s: &str| {
                    s.trim()
                        .replace(',', ".")
                        .parse::<f64>()
                        .ok()
                        .filter(|n| n.is_finite())
                };
                let (a, b) = (parse(&self.a), parse(&self.b));
                // A handled UI event is acknowledged; arithmetic errors are in the rendered alert.
                let _ = self.calculate(key, &json!({"a":a,"b":b}));
                return Ok(());
            }
            "target-a" => self.target_b = false,
            "target-b" => self.target_b = true,
            "clear" => *self = Self::default(),
            "backspace" => {
                self.active().pop();
            }
            "sign" => {
                let s = self.active();
                if s.starts_with('-') {
                    s.remove(0);
                } else {
                    s.insert(0, '-');
                }
            }
            "decimal" => {
                let s = self.active();
                if !s.contains(['.', ',']) {
                    if s.is_empty() || s == "-" {
                        s.push('0');
                    }
                    s.push('.');
                }
            }
            _ if key.starts_with("digit-")
                && key.len() == 7
                && key.as_bytes()[6].is_ascii_digit() =>
            {
                let s = self.active();
                if s.len() < 128 {
                    s.push(key.as_bytes()[6] as char);
                }
            }
            _ => return Err("INVALID_INPUT"),
        }
        self.error = None;
        self.result = None;
        Ok(())
    }
    fn active(&mut self) -> &mut String {
        if self.target_b {
            &mut self.b
        } else {
            &mut self.a
        }
    }
    pub fn render(&self) -> Value {
        let mut children = vec![
            node(
                "heading",
                "title",
                json!({"text":"Calculatrice","level":2}),
                vec![],
            ),
            node(
                "input",
                "a",
                json!({"label":"Premier nombre","value":self.a}),
                vec![],
            ),
            node(
                "input",
                "b",
                json!({"label":"Deuxième nombre","value":self.b}),
                vec![],
            ),
            node(
                "text",
                "target",
                json!({"text":if self.target_b {"Pavé numérique : deuxième nombre"}else{"Pavé numérique : premier nombre"}}),
                vec![],
            ),
            node(
                "row",
                "targets",
                json!({"gap":8}),
                vec![
                    button("target-a", "Saisir le premier nombre"),
                    button("target-b", "Saisir le deuxième nombre"),
                ],
            ),
            node(
                "grid",
                "operations",
                json!({"columns":4}),
                vec![
                    button("add", "Additionner"),
                    button("subtract", "Soustraire"),
                    button("multiply", "Multiplier"),
                    button("divide", "Diviser"),
                ],
            ),
        ];
        let mut keys: Vec<_> = [7, 8, 9, 4, 5, 6, 1, 2, 3, 0]
            .iter()
            .map(|n| button(&format!("digit-{n}"), &n.to_string()))
            .collect();
        keys.extend([
            button("decimal", "Virgule"),
            button("sign", "Changer le signe"),
            button("backspace", "Effacer un chiffre"),
            button("clear", "Tout effacer"),
        ]);
        children.push(node("grid", "keypad", json!({"columns":3}), keys));
        let (text, severity) = if let Some(code) = self.error {
            (code.to_string(), "error")
        } else if let Some(result) = self.result {
            (format!("Résultat : {result}"), "success")
        } else {
            ("Prêt à calculer".into(), "info")
        };
        children.push(node(
            "alert",
            "result",
            json!({"text":text,"severity":severity}),
            vec![],
        ));
        node("column", "calculator", json!({"gap":12}), children)
    }
}
