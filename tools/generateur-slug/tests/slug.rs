use kortex_sdk::{Request, Tool};
use serde_json::{Value, json};
use tool_generateur_slug::{SlugGenerator, convert};
fn call(tool: &mut SlugGenerator, kind: &str, payload: Value) -> kortex_sdk::Reply {
    tool.handle(serde_json::from_value::<Request>(json!({"v":1,"request_id":"12070000-0000-4000-8000-000000000001","kind":kind,"payload":payload})).unwrap())
}
fn run(action: &str, text: &str) -> Result<String, &'static str> {
    convert(action, &json!({"text":text})).map(|v| v["text"].as_str().unwrap().to_string())
}
#[test]
fn builds_ascii_slugs() {
    assert_eq!(
        run("slug", "L’Été à Genève : Œuvres & Straße !").unwrap(),
        "l-ete-a-geneve-oeuvres-strasse"
    );
    assert_eq!(
        run("snake", "  Hello,  World -- 42  ").unwrap(),
        "hello_world_42"
    );
    assert_eq!(run("slug", "e\u{301}cole").unwrap(), "ecole");
    assert_eq!(run("slug", "İstanbul ŁÓDŹ").unwrap(), "istanbul-lodz");
    assert_eq!(run("slug", "--a--").unwrap(), "a");
    assert_eq!(run("slug", "a😀b").unwrap(), "a-b");
}
#[test]
fn rejects_empty_results_and_bad_input() {
    assert_eq!(run("slug", ""), Err("EMPTY_RESULT"));
    assert_eq!(run("slug", "日本語 !!"), Err("EMPTY_RESULT"));
    assert_eq!(run("slug", &"a".repeat(2049)), Err("TEXT_TOO_LARGE"));
    assert_eq!(run("slug", &"a".repeat(2048)).unwrap().len(), 2048);
    assert_eq!(
        convert("slug", &json!({"text":"a","x":1})),
        Err("INVALID_INPUT")
    );
    assert_eq!(convert("nope", &json!({"text":"a"})), Err("UNKNOWN_ACTION"));
}
#[test]
fn reactive_ui_matches_the_action() {
    let mut tool = SlugGenerator::default();
    assert!(call(&mut tool, "init", json!({})).ok);
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"text","event_type":"input","value":"Été 2026"})
        )
        .ok
    );
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"slug","event_type":"click","value":null})
        )
        .ok
    );
    let tree = Value::Object(call(&mut tool, "render", json!({})).data.unwrap());
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    assert_eq!(tree["children"][3]["props"]["value"], "ete-2026");
    assert!(call(&mut tool, "shutdown", json!({})).ok);
    assert!(!call(&mut tool, "render", json!({})).ok);
}
