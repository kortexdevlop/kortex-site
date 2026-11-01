use kortex_sdk::{Request, Tool};
use serde_json::{Value, json};
use tool_formateur_json::{JsonFormatter, convert};
fn call(tool: &mut JsonFormatter, kind: &str, payload: Value) -> kortex_sdk::Reply {
    tool.handle(serde_json::from_value::<Request>(json!({"v":1,"request_id":"12050000-0000-4000-8000-000000000001","kind":kind,"payload":payload})).unwrap())
}
#[test]
fn formats_and_minifies() {
    let src = r#"{"b":[1,2],"a":"é"}"#;
    assert_eq!(
        convert("format", &json!({"text":src})).unwrap()["text"],
        "{\n  \"a\": \"é\",\n  \"b\": [\n    1,\n    2\n  ]\n}"
    );
    assert_eq!(
        convert("minify", &json!({"text":" { \"a\" : [ 1 , 2 ] } "})).unwrap()["text"],
        r#"{"a":[1,2]}"#
    );
}
#[test]
fn rejects_invalid_and_oversized() {
    assert_eq!(
        convert("format", &json!({"text":"{a:1}"})),
        Err("INVALID_JSON")
    );
    assert_eq!(convert("format", &json!({"text":""})), Err("INVALID_JSON"));
    assert_eq!(
        convert("format", &json!({"text":"1","x":1})),
        Err("INVALID_INPUT")
    );
    assert_eq!(convert("nope", &json!({"text":"1"})), Err("UNKNOWN_ACTION"));
    assert_eq!(
        convert("format", &json!({"text":"1".repeat(32769)})),
        Err("TEXT_TOO_LARGE")
    );
    let wide = format!("[{}]", vec!["1"; 8000].join(","));
    assert!(convert("format", &json!({"text":wide})).is_ok());
    let nested = format!("{}1{}", "[".repeat(100), "]".repeat(100));
    assert!(convert("format", &json!({"text":nested})).is_ok());
}
#[test]
fn reactive_ui_matches_the_action() {
    let mut tool = JsonFormatter::default();
    assert!(call(&mut tool, "init", json!({})).ok);
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"text","event_type":"input","value":"{\"a\":1}"})
        )
        .ok
    );
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"format","event_type":"click","value":null})
        )
        .ok
    );
    let tree = Value::Object(call(&mut tool, "render", json!({})).data.unwrap());
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    assert_eq!(
        tree["children"][3]["props"]["value"],
        convert("format", &json!({"text":"{\"a\":1}"})).unwrap()["text"]
    );
    assert!(call(&mut tool, "shutdown", json!({})).ok);
    assert!(!call(&mut tool, "render", json!({})).ok);
}
