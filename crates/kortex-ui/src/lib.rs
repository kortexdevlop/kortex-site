//! Host-owned UI: frozen schema validation precedes creation of any guest-driven nodes.
pub mod chart;
pub mod components;
pub mod events;
use dioxus::prelude::*;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
};
pub const SHELL_CSS: &str = include_str!("../../../runtime/assets/tool-shell.css");
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiError {
    Limit,
    Schema,
    DuplicateKey,
    ChartLength,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedUiTree(Value);
impl ValidatedUiTree {
    pub fn parse(bytes: &[u8]) -> Result<Self, UiError> {
        if bytes.len() > 1024 * 1024 {
            return Err(UiError::Limit);
        }
        Self::validate(serde_json::from_slice(bytes).map_err(|_| UiError::Schema)?)
    }
    pub fn validate(value: Value) -> Result<Self, UiError> {
        // Bound arbitrary JSON nesting before serialization or recursive schema evaluation.
        let mut pending = vec![(&value, 0)];
        let mut visited = 0;
        while let Some((part, depth)) = pending.pop() {
            visited += 1;
            if depth > 128 || visited > 1024 * 1024 {
                return Err(UiError::Limit);
            }
            match part {
                Value::Array(items) => pending.extend(items.iter().map(|v| (v, depth + 1))),
                Value::Object(items) => pending.extend(items.values().map(|v| (v, depth + 1))),
                _ => {}
            }
        }
        // Bound depth/count before recursive schema validation.
        let mut stack = vec![(&value, 0)];
        let mut keys = BTreeSet::new();
        let mut count = 0;
        while let Some((node, depth)) = stack.pop() {
            count += 1;
            if depth > 32 || count > 2000 {
                return Err(UiError::Limit);
            }
            if let Some(key) = node["key"].as_str() {
                if !keys.insert(key) {
                    return Err(UiError::DuplicateKey);
                }
            }
            if let Some(children) = node["children"].as_array() {
                stack.extend(children.iter().map(|n| (n, depth + 1)));
            }
        }
        if serde_json::to_vec(&value)
            .map_err(|_| UiError::Schema)?
            .len()
            > 1024 * 1024
        {
            return Err(UiError::Limit);
        }
        static SCHEMA: OnceLock<jsonschema::Validator> = OnceLock::new();
        let validator = SCHEMA.get_or_init(|| {
            jsonschema::validator_for(
                &serde_json::from_str(include_str!("../../../shared/schemas/ui.schema.json"))
                    .expect("bundled UI schema"),
            )
            .expect("compiled UI schema")
        });
        if !validator.is_valid(&value) {
            return Err(UiError::Schema);
        }
        let mut stack = vec![&value];
        while let Some(node) = stack.pop() {
            if node["type"] == "svg-chart" {
                let n = node["props"]["labels"].as_array().unwrap().len();
                if node["props"]["series"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|s| s["values"].as_array().unwrap().len() != n)
                {
                    return Err(UiError::ChartLength);
                }
            }
            stack.extend(node["children"].as_array().unwrap());
        }
        Ok(Self(value))
    }
}
/// Host-generated prefix prevents label/id collisions between mounted tools.
#[component]
pub fn UiRenderer(ui: ValidatedUiTree, on_event: Callback<events::UiEvent>) -> Element {
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    let prefix = use_hook(|| format!("kortex-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
    rsx! { div { class: "kortex-tool", components::RenderNode { node:ui.0,prefix,on_event } } }
}
/// Boundary for untrusted input: invalid trees produce a host-owned visible error.
#[component]
pub fn UiBoundary(value: Value, on_event: Callback<events::UiEvent>) -> Element {
    match ValidatedUiTree::validate(value) {
        Ok(ui) => rsx! { UiRenderer {ui,on_event} },
        Err(_) => rsx! { p { class:"kortex-error",role:"alert","Interface de l’outil invalide." } },
    }
}
