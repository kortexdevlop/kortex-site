use serde_json::{Value, json};
#[derive(Debug, Clone, PartialEq)]
pub struct UiEvent {
    node_key: String,
    event_type: &'static str,
    value: Value,
}
impl UiEvent {
    pub(crate) fn new(node_key: &str, event_type: &'static str, value: Value) -> Self {
        Self {
            node_key: node_key.into(),
            event_type,
            value,
        }
    }
    pub fn payload(&self) -> Value {
        json!({"node_key":self.node_key,"event_type":self.event_type,"value":self.value})
    }
}
/// Native input values stay strings, including input_type=number, as allowed by the frozen event schema.
pub(crate) fn input(key: &str, value: String) -> UiEvent {
    UiEvent::new(key, "input", Value::String(value))
}
pub(crate) fn change(key: &str, value: Value) -> UiEvent {
    UiEvent::new(key, "change", value)
}
pub fn next_tab(current: usize, count: usize, key: &str) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        "ArrowRight" | "ArrowDown" => Some((current + 1) % count),
        "ArrowLeft" | "ArrowUp" => Some((current + count - 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}
