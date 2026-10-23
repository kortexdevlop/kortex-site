use dioxus::prelude::*;
use serde_json::Value;
const STYLE: &str = include_str!("../assets/style.css");
fn main() {
    dioxus::launch(App);
}
#[component]
fn App() -> Element {
    let tree: Value =
        serde_json::from_str(include_str!("../../../../shared/fixtures/ui.valid.json"))
            .expect("validated fixture");
    rsx! {
     style { "{STYLE}" }
     main { aria_label:"Kortex renderer prototype",
      h1 { "Kortex renderer prototype" }
      p { class:"intro", "One JSON tree · Dioxus host rendering · no guest DOM" }
      section { class:"card", aria_label:"Fixture card", RenderNode {node:tree} }
      section { class:"card", aria_label:"Table fixture",
       h2 { "Table" }
       table { thead { tr { th { "Label" } th { "Value" } } } tbody {tr {td {"A"} td {"2"}} tr {td {"B"} td {"4"}}} }
      }
     }
    }
}
#[component]
fn RenderNode(node: Value) -> Element {
    let key = node["key"].as_str().unwrap_or("").to_string();
    let children = node["children"].as_array().cloned().unwrap_or_default();
    let props = &node["props"];
    let text_value = props["text"].as_str().unwrap_or("").to_string();
    match node["type"].as_str().unwrap_or("") {
        "column" => {
            rsx! {div {class:"column", "data-key":key, for child in children {RenderNode {node:child}}}}
        }
        "heading" => rsx! {h2 {"data-key":key, "{text_value}"}},
        "text" => rsx! {p {"data-key":key,"{text_value}"}},
        "input" => {
            rsx! {InputField {key_id:key,label:props["label"].as_str().unwrap_or("").to_string(),initial:props["value"].as_str().unwrap_or("").to_string()}}
        }
        "button" => {
            rsx! {ActionButton {key_id:key,text:props["text"].as_str().unwrap_or("").to_string()}}
        }
        "svg-chart" => {
            let labels = props["labels"].as_array().cloned().unwrap_or_default();
            let values = props["series"][0]["values"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let bars: Vec<_> = values
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    (
                        40 + i * 130,
                        140 - v.as_i64().unwrap_or(0) * 25,
                        v.as_i64().unwrap_or(0) * 25,
                        labels
                            .get(i)
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    )
                })
                .collect();
            rsx! {figure {"data-key":key, figcaption {"Values"}
             svg {view_box:"0 0 400 180",role:"img","aria-label":"Values bar chart",width:"400",height:"180",
              for (x,y,height,label) in bars {
               rect {x:"{x}",y:"{y}",width:"72",height:"{height}",fill:"#375bd2"}
               text {x:"{x}",y:"165",fill:"#182140", "{label}"}
              }
             }
            }}
        }
        _ => rsx! {p {role:"alert","Unsupported prototype node"}},
    }
}
#[component]
fn InputField(key_id: String, label: String, initial: String) -> Element {
    let mut value = use_signal(|| initial);
    rsx! {div {class:"field",label {r#for:key_id.clone(),"{label}"} input {id:key_id.clone(),"data-key":key_id,value:"{value}",oninput:move |event|value.set(event.value())} output {id:"input-state", "Input: {value}"}}}
}
#[component]
fn ActionButton(key_id: String, text: String) -> Element {
    let mut clicks = use_signal(|| 0);
    rsx! {div {class:"action",button {"data-key":key_id,onclick:move |_|clicks+=1,"{text}"} output {id:"click-state", "Clicks: {clicks}"}}}
}
