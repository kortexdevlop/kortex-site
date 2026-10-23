use dioxus::core::Mutation;
use dioxus::prelude::*;
use kortex_ui::{
    UiBoundary, UiError, UiRenderer, ValidatedUiTree, components::NODE_TYPES, events::next_tab,
};
use serde_json::{Value, json};
fn node(kind: &str, key: &str, props: Value) -> Value {
    json!({"type":kind,"key":key,"props":props,"children":[]})
}
fn fixture() -> Value {
    let props = [
        json!({"gap":8}),
        json!({"gap":4}),
        json!({"columns":2}),
        json!({"title":"Carte"}),
        json!({"text":"Texte"}),
        json!({"text":"Titre","level":1}),
        json!({"text":"Exécuter"}),
        json!({"label":"Nom","value":"Alice"}),
        json!({"label":"Notes","value":"Bonjour"}),
        json!({"label":"Choix","options":[{"label":"Un","value":"1"}],"value":"1"}),
        json!({"label":"Accord","checked":true}),
        json!({"columns":[{"key":"name","label":"Nom"}],"rows":[{"name":"Alice"}]}),
        json!({"label":"Avancement","value":50}),
        json!({"text":"Information","severity":"info"}),
        json!({"kind":"bar","labels":["A","B"],"series":[{"label":"Série","values":[-2,4]}]}),
        json!({"label":"Actions"}),
        json!({"label":"Sections","active_key":"panelA"}),
    ];
    let mut children: Vec<Value> = NODE_TYPES
        .iter()
        .zip(props)
        .enumerate()
        .map(|(i, (kind, p))| node(kind, &format!("node{i}"), p))
        .collect();
    children.last_mut().unwrap()["children"] = json!([
        node("text", "panelA", json!({"text":"Premier"})),
        node("text", "panelB", json!({"text":"Second"}))
    ]);
    json!({"type":"column","key":"root","props":{},"children":children})
}
fn html(value: Value) -> String {
    {
        fn root(value: Value) -> Element {
            rsx! {UiBoundary {value,on_event:move |_|{}}}
        }
        let mut dom = VirtualDom::new_with_props(root, value);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }
}
#[test]
fn all_frozen_types_and_snapshot() {
    let schema: Value =
        serde_json::from_str(include_str!("../../../shared/schemas/ui.schema.json")).unwrap();
    let types: Vec<_> = schema["$defs"]["node"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["properties"]["type"]["const"].as_str().unwrap())
        .collect();
    assert_eq!(types, NODE_TYPES);
    let rendered = html(fixture());
    for kind in NODE_TYPES {
        assert!(
            rendered.contains(&format!("data-kind=\"{kind}\"")),
            "{kind}"
        );
    }
    // Stable tree snapshot independent of host-generated instance ids.
    let snapshot = rendered
        .split("data-kind=\"")
        .skip(1)
        .map(|p| p.split('"').next().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(snapshot, include_str!("snapshots/tree.txt").trim());
    assert!(!rendered.contains("Interface de l’outil invalide"));
    if let Ok(path) = std::env::var("KORTEX_UI_PROOF_HTML") {
        std::fs::write(path,format!("<!doctype html><html lang=\"fr\"><head><meta charset=\"utf-8\"><title>Kortex UI</title><style>{}</style></head><body><main>{rendered}</main></body></html>",kortex_ui::SHELL_CSS)).unwrap();
    }
}
#[test]
fn accessible_labels_native_tab_order_and_tabs() {
    let rendered = html(fixture());
    for label in ["Nom", "Notes", "Choix", "Accord", "Avancement"] {
        assert!(rendered.contains(&format!("aria-label=\"{label}\"")));
    }
    assert!(rendered.contains("role=\"tablist\""));
    assert!(rendered.contains("role=\"tabpanel\""));
    assert!(rendered.contains("aria-selected=\"true\""));
    assert!(rendered.contains("aria-selected=\"false\""));
    assert!(rendered.contains("tabindex=-1"));
    assert!(!rendered.contains("tabindex=1"));
    assert!(rendered.contains("aria-controls="));
    assert!(rendered.contains("aria-labelledby="));
    for (key, expected) in [("ArrowRight", 1), ("ArrowLeft", 2), ("Home", 0), ("End", 2)] {
        assert_eq!(next_tab(0, 3, key), Some(expected));
    }
    assert_eq!(next_tab(0, 0, "Home"), None);
    assert_eq!(next_tab(0, 3, "Tab"), None);
}
#[test]
fn html_injection_is_text_in_every_content_path() {
    let attack = "<img src=x onerror=alert(1)>";
    for kind in ["text", "heading", "button", "alert"] {
        let mut p = json!({"text":attack});
        if kind == "heading" {
            p["level"] = json!(2);
        }
        if kind == "alert" {
            p["severity"] = json!("error");
        }
        let rendered = html(node(kind, "attack", p));
        assert!(!rendered.contains("<img"));
        assert!(rendered.contains("&#60;img"), "{rendered}");
    }
    let rendered = html(node(
        "table",
        "attack",
        json!({"columns":[{"key":"a","label":attack}],"rows":[{"a":attack}]}),
    ));
    assert!(!rendered.contains("<img"));
    assert!(rendered.contains("&#60;img"), "{rendered}");
    let rendered = html(node(
        "svg-chart",
        "attack",
        json!({"kind":"line","labels":[attack],"series":[{"label":attack,"values":[1]}]}),
    ));
    assert!(!rendered.contains("<img"));
    assert!(rendered.contains("&#60;img"), "{rendered}");
    assert!(!rendered.contains("NaN"));
    assert!(!rendered.contains("Infinity"));
}
#[test]
fn invalid_trees_have_visible_fallback_without_partial_render() {
    for kind in ["script", "iframe", "unknown"] {
        let rendered = html(node(kind, "bad", json!({})));
        assert!(rendered.contains("role=\"alert\""));
        assert!(!rendered.contains("data-kind="));
    }
    let mut tree = fixture();
    tree["children"][1]["key"] = tree["children"][0]["key"].clone();
    assert_eq!(ValidatedUiTree::validate(tree), Err(UiError::DuplicateKey));
    let mut tree = node("text", "root", json!({"text":"leaf"}));
    for i in 0..33 {
        tree = json!({"type":"column","key":format!("parent{i}"),"props":{},"children":[tree]});
    }
    assert_eq!(ValidatedUiTree::validate(tree), Err(UiError::Limit));
    let mut tree = fixture();
    tree["children"][14]["props"]["series"][0]["values"] = json!([1]);
    assert_eq!(ValidatedUiTree::validate(tree), Err(UiError::ChartLength));
    assert_eq!(
        ValidatedUiTree::parse(&vec![b' '; 1024 * 1024 + 1]),
        Err(UiError::Limit)
    );
    let tree = json!({"type":"column","key":"root","props":{},"children":(0..2000).map(|i|node("text",&format!("n{i}"),json!({"text":"x"}))).collect::<Vec<_>>()});
    assert_eq!(ValidatedUiTree::validate(tree), Err(UiError::Limit));
}
#[test]
fn headings_charts_disabled_and_multiple_mounts() {
    for level in 1..=6 {
        assert!(
            html(node("heading", "h", json!({"text":"Titre","level":level})))
                .contains(&format!("<h{level}>"))
        );
    }
    let rendered = html(node("button", "b", json!({"text":"Stop","disabled":true})));
    assert!(rendered.contains("disabled"));
    for kind in ["line", "bar"] {
        for values in [
            json!([]),
            json!([0, 0]),
            json!([-9007199254740991i64, 9007199254740991i64]),
        ] {
            let labels: Vec<_> = values
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, _)| format!("{i}"))
                .collect();
            let rendered = html(node(
                "svg-chart",
                "chart",
                json!({"kind":kind,"labels":labels,"series":[{"label":"S","values":values}]}),
            ));
            assert!(!rendered.contains("NaN"));
            assert!(!rendered.contains("Infinity"));
            assert!(rendered.contains("Valeurs du graphique"));
        }
    }
    let ui = ValidatedUiTree::validate(node("input", "same", json!({"label":"Nom"}))).unwrap();
    fn pair(ui: ValidatedUiTree) -> Element {
        rsx! {UiRenderer {ui:ui.clone(),on_event:move |_|{}} UiRenderer {ui,on_event:move |_|{}}}
    }
    let mut dom = VirtualDom::new_with_props(pair, ui);
    dom.rebuild_in_place();
    let rendered = dioxus_ssr::render(&dom);
    let ids: Vec<_> = rendered
        .split("id=\"")
        .skip(1)
        .map(|s| s.split('"').next().unwrap())
        .collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1]);
}

#[test]
#[allow(deprecated)]
fn actual_dioxus_handlers_emit_schema_valid_events_and_tabs_switch() {
    use dioxus_html::{
        Code, EventData, Key, Location, Modifiers, SerializedFormData,
        SerializedHtmlEventConverter, SerializedKeyboardData, SerializedMouseData,
        set_event_converter,
    };
    use std::{cell::RefCell, rc::Rc};
    set_event_converter(Box::new(SerializedHtmlEventConverter));
    type Output = Rc<RefCell<Vec<Value>>>;
    fn root((value, output): (Value, Output)) -> Element {
        rsx! {UiBoundary {value,on_event:move |e:kortex_ui::events::UiEvent|output.borrow_mut().push(e.payload())}}
    }
    let schema: Value =
        serde_json::from_str(include_str!("../../../shared/schemas/ui.schema.json")).unwrap();
    let validator =
        jsonschema::validator_for(&json!({"$ref":"#/$defs/event","$defs":schema["$defs"]}))
            .unwrap();
    let cases = [
        (
            "button",
            json!({"text":"Go"}),
            "click",
            EventData::Mouse(SerializedMouseData::default()),
            json!(null),
            "click",
        ),
        (
            "input",
            json!({"label":"Nom"}),
            "input",
            EventData::Form(SerializedFormData::new(
                "<script>texte</script>".into(),
                vec![],
            )),
            json!("<script>texte</script>"),
            "input",
        ),
        (
            "textarea",
            json!({"label":"Notes"}),
            "input",
            EventData::Form(SerializedFormData::new("Texte".into(), vec![])),
            json!("Texte"),
            "input",
        ),
        (
            "select",
            json!({"label":"Choix","options":[{"value":"a","label":"A"}]}),
            "change",
            EventData::Form(SerializedFormData::new("a".into(), vec![])),
            json!("a"),
            "change",
        ),
        (
            "checkbox",
            json!({"label":"Accord","checked":false}),
            "change",
            EventData::Form(SerializedFormData::new("true".into(), vec![])),
            json!(true),
            "change",
        ),
    ];
    for (kind, props, name, data, expected, event_type) in cases {
        let output = Output::default();
        let mut dom =
            VirtualDom::new_with_props(root, (node(kind, "target", props), output.clone()));
        let mutations = dom.rebuild_to_vec();
        let target = mutations
            .edits
            .iter()
            .find_map(|m| match m {
                Mutation::NewEventListener { name: n, id } if n == name => Some(*id),
                _ => None,
            })
            .unwrap();
        dom.handle_event(name, data.into_any(), target, true);
        let events = output.borrow();
        assert_eq!(events.len(), 1, "{kind}");
        assert!(validator.is_valid(&events[0]));
        assert_eq!(
            events[0],
            json!({"node_key":"target","event_type":event_type,"value":expected})
        );
    }
    let output = Output::default();
    let mut dom = VirtualDom::new_with_props(
        root,
        (
            node("button", "disabled", json!({"text":"Stop","disabled":true})),
            output.clone(),
        ),
    );
    let mutations = dom.rebuild_to_vec();
    let target = mutations
        .edits
        .iter()
        .find_map(|m| match m {
            Mutation::NewEventListener { name, id } if name == "click" => Some(*id),
            _ => None,
        })
        .unwrap();
    dom.handle_event(
        "click",
        EventData::Mouse(SerializedMouseData::default()).into_any(),
        target,
        true,
    );
    assert!(output.borrow().is_empty());
    let mut tabs = node("tabs", "tabs", json!({"label":"Sections"}));
    tabs["children"] = json!([
        node("text", "first", json!({"text":"Premier"})),
        node("text", "second", json!({"text":"Second"}))
    ]);
    let output = Output::default();
    let mut dom = VirtualDom::new_with_props(root, (tabs, output.clone()));
    let mutations = dom.rebuild_to_vec();
    let target = mutations
        .edits
        .iter()
        .find_map(|m| match m {
            Mutation::NewEventListener { name, id } if name == "keydown" => Some(*id),
            _ => None,
        })
        .unwrap();
    dom.handle_event(
        "keydown",
        EventData::Keyboard(SerializedKeyboardData::new(
            Key::ArrowRight,
            Code::ArrowRight,
            Location::Standard,
            false,
            Modifiers::empty(),
            false,
        ))
        .into_any(),
        target,
        true,
    );
    dom.render_immediate_to_vec();
    assert_eq!(
        output.borrow().last().unwrap(),
        &json!({"node_key":"tabs","event_type":"select","value":"second"})
    );
    assert!(validator.is_valid(output.borrow().last().unwrap()));
    let rendered = dioxus_ssr::render(&dom);
    assert!(
        rendered.contains("aria-selected=\"true\" tabindex=0>Second"),
        "{rendered}"
    );
}

#[test]
fn every_type_rejects_unknown_properties_and_escapes_labels() {
    for mut child in fixture()["children"].as_array().unwrap().clone() {
        let kind = child["type"].as_str().unwrap().to_string();
        child["props"]["dangerously_set_inner_html"] = json!("<script>evil()</script>");
        let rendered = html(child);
        assert!(rendered.contains("role=\"alert\""), "{kind}");
        assert!(!rendered.contains("data-kind="));
    }
    let attack = "<img src=x onerror=alert(1)>";
    for mut child in fixture()["children"].as_array().unwrap().clone() {
        for name in ["text", "label", "aria_label", "title", "placeholder"] {
            if child["props"].get(name).is_some() {
                child["props"][name] = json!(attack);
            }
        }
        let rendered = html(child);
        assert!(!rendered.contains("<img"));
        assert!(!rendered.contains("Interface de l’outil invalide"));
    }
}
