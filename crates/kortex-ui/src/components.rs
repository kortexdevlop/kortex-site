use crate::{
    chart::Chart,
    events::{self, UiEvent},
};
use dioxus::prelude::*;
use serde_json::Value;
pub const NODE_TYPES: [&str; 17] = [
    "column",
    "row",
    "grid",
    "card",
    "text",
    "heading",
    "button",
    "input",
    "textarea",
    "select",
    "checkbox",
    "table",
    "progress",
    "alert",
    "svg-chart",
    "toolbar",
    "tabs",
];
fn string(p: &Value, k: &str) -> String {
    p[k].as_str().unwrap_or("").into()
}
#[component]
pub(crate) fn RenderNode(node: Value, prefix: String, on_event: Callback<UiEvent>) -> Element {
    let kind = node["type"].as_str().unwrap_or("");
    let p = &node["props"];
    let key = string(&node, "key");
    let id = format!("{prefix}-control-{key}");
    let children = node["children"].as_array().cloned().unwrap_or_default();
    let content = rsx! {for child in children.iter() { RenderNode {key:"{child[\"key\"].as_str().unwrap()}",node:child.clone(),prefix:prefix.clone(),on_event} }};
    let label = string(p, "label");
    let text = string(p, "text");
    let aria = p["aria_label"]
        .as_str()
        .unwrap_or(if label.is_empty() { &text } else { &label })
        .to_string();
    let disabled = p["disabled"].as_bool().unwrap_or(false);
    let value = string(p, "value");
    // Every schema type has an explicit arm; only host-generated attributes/elements are used.
    let element = match kind {
        "column" | "row" | "grid" => {
            let class = format!("kortex-{kind}");
            let style = if kind == "grid" {
                format!(
                    "grid-template-columns:repeat({},minmax(0,1fr))",
                    p["columns"].as_u64().unwrap_or(1)
                )
            } else {
                format!("gap:{}px", p["gap"].as_u64().unwrap_or(12))
            };
            rsx! {div {class,style,{content}}}
        }
        "card" => {
            rsx! {section {class:"kortex-card", if let Some(title)=p["title"].as_str() {h2 {"{title}"}} {content}}}
        }
        "text" => rsx! {p {"{text}"}{content}},
        "heading" => match p["level"].as_u64().unwrap_or(2) {
            1 => rsx! {h1 {"{text}"}{content}},
            2 => rsx! {h2 {"{text}"}{content}},
            3 => rsx! {h3 {"{text}"}{content}},
            4 => rsx! {h4 {"{text}"}{content}},
            5 => rsx! {h5 {"{text}"}{content}},
            _ => rsx! {h6 {"{text}"}{content}},
        },
        "button" => {
            rsx! {button {r#type:"button",aria_label:aria,disabled,onclick:move |_|{if !disabled {on_event.call(UiEvent::new(&key,"click",Value::Null))}},"{text}"}{content}}
        }
        "input" => {
            let input_type = p["input_type"].as_str().unwrap_or("text");
            let placeholder = string(p, "placeholder");
            rsx! {div {class:"kortex-field",label {r#for:id.clone(),"{label}"} input {id,r#type:input_type,aria_label:aria,disabled,value,placeholder,oninput:move |e|{if !disabled {on_event.call(events::input(&key,e.value()))}}}}{content}}
        }
        "textarea" => {
            rsx! {div {class:"kortex-field",label {r#for:id.clone(),"{label}"} textarea {id,aria_label:aria,disabled,value:value.clone(),oninput:move |e|{if !disabled {on_event.call(events::input(&key,e.value()))}},"{value}"}}{content}}
        }
        "select" => {
            let options = p["options"].as_array().unwrap();
            rsx! {div {class:"kortex-field",label {r#for:id.clone(),"{label}"} select {id,aria_label:aria,disabled,value:value.clone(),onchange:move |e|{if !disabled {on_event.call(events::change(&key,Value::String(e.value())))}} ,for option in options {option {value:string(option,"value"),selected:string(option,"value")==value,{string(option,"label")}}}}}{content}}
        }
        "checkbox" => {
            rsx! {div {class:"kortex-field",label {r#for:id.clone(),"{label}"} input {id,r#type:"checkbox",aria_label:aria,disabled,checked:p["checked"].as_bool().unwrap(),onchange:move |e|{if !disabled {on_event.call(events::change(&key,Value::Bool(e.checked())))}}}}{content}}
        }
        "table" => {
            let columns = p["columns"].as_array().unwrap();
            let rows = p["rows"].as_array().unwrap();
            rsx! {table {caption {"Tableau de données"} thead {tr {for col in columns {th {scope:"col",{string(col,"label")}}}}} tbody {for row in rows {tr {for col in columns {td {{cell(&row[&string(col,"key")])}}}}}}}{content}}
        }
        "progress" => {
            rsx! {div {class:"kortex-field",label {r#for:id.clone(),"{label}"} progress {id,aria_label:aria,max:"100",value:p["value"].as_f64().unwrap(),{p["value"].to_string()} "%"}}{content}}
        }
        "alert" => {
            let severity = string(p, "severity");
            let role = if severity == "error" || severity == "warning" {
                "alert"
            } else {
                "status"
            };
            rsx! {div {class:"kortex-alert {severity}",role,"{text}"}{content}}
        }
        "svg-chart" => rsx! {Chart {data:p.clone()}{content}},
        // A labelled group keeps all controls in native tab order; no partial toolbar arrow model.
        "toolbar" => {
            rsx! {div {class:"kortex-row",role:"group",aria_label:if label.is_empty(){"Outils".to_string()}else{label},{content}}}
        }
        "tabs" => {
            let revision = format!(
                "{}:{}",
                p["active_key"],
                children
                    .iter()
                    .map(|c| string(c, "key"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            rsx! {Tabs {key:"{revision}",node:node.clone(),prefix:prefix.clone(),on_event}}
        }
        _ => rsx! {p {role:"alert","Composant non pris en charge."}},
    };
    rsx! {div {"data-node":node["key"].as_str().unwrap_or(""),"data-kind":kind,{element}}}
}
fn cell(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        _ => v.to_string(),
    }
}
#[component]
fn Tabs(node: Value, prefix: String, on_event: Callback<UiEvent>) -> Element {
    let children = node["children"].as_array().unwrap();
    let initial = node["props"]["active_key"]
        .as_str()
        .and_then(|k| children.iter().position(|c| c["key"] == k))
        .unwrap_or(0);
    let mut selected = use_signal(|| initial);
    let mut mounted = use_signal(Vec::<Option<MountedEvent>>::new);
    let index = (*selected.read()).min(children.len().saturating_sub(1));
    let disabled = node["props"]["disabled"].as_bool().unwrap_or(false);
    let label = node["props"]["aria_label"]
        .as_str()
        .or(node["props"]["label"].as_str())
        .unwrap_or("Onglets")
        .to_string();
    let parent_key = string(&node, "key");
    let child_keys: Vec<String> = children.iter().map(|c| string(c, "key")).collect();
    rsx! {div {
        div {role:"tablist",aria_label:label,
            for (i,child) in children.iter().enumerate() {
                button {key:"{child[\"key\"].as_str().unwrap()}",r#type:"button",role:"tab",id:format!("{prefix}-tab-{}",child["key"].as_str().unwrap()),aria_controls:format!("{prefix}-panel-{}",child["key"].as_str().unwrap()),aria_selected:(i==index).to_string(),tabindex:if i==index {0}else{-1},disabled,
                    onmounted:move |e| {let mut nodes=mounted.write();if nodes.len()<=i {nodes.resize(i+1,None);}nodes[i]=Some(e);},
                    onclick:{let keys=child_keys.clone();let parent=parent_key.clone();move |_|{if !disabled {selected.set(i);on_event.call(UiEvent::new(&parent,"select",Value::String(keys[i].clone())));}}},
                    onkeydown:{let keys=child_keys.clone();let parent=parent_key.clone();move |e:KeyboardEvent|{
                        if disabled {return;}
                        if let Some(next)=events::next_tab(i,keys.len(),&e.key().to_string()) {
                            e.prevent_default();selected.set(next);on_event.call(UiEvent::new(&parent,"select",Value::String(keys[next].clone())));
                            let target=mounted.read().get(next).and_then(Clone::clone);
                            if let Some(target)=target {spawn(async move {let _=target.set_focus(true).await;});}
                        }
                    }},
                    {child["props"]["title"].as_str().or(child["props"]["label"].as_str()).or(child["props"]["text"].as_str()).unwrap_or(child["key"].as_str().unwrap())}
                }
            }
        }
        for (i,child) in children.iter().enumerate() {div {key:"{child[\"key\"].as_str().unwrap()}",role:"tabpanel",id:format!("{prefix}-panel-{}",child["key"].as_str().unwrap()),aria_labelledby:format!("{prefix}-tab-{}",child["key"].as_str().unwrap()),hidden:i!=index,tabindex:0,RenderNode {node:child.clone(),prefix:prefix.clone(),on_event}}}
    }}
}
