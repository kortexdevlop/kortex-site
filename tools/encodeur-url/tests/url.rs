use kortex_sdk::{Request, Tool};
use serde_json::{Value, json};
use tool_encodeur_url::{UrlEncoder, convert};
fn call(tool: &mut UrlEncoder, kind: &str, payload: Value) -> kortex_sdk::Reply {
    tool.handle(serde_json::from_value::<Request>(json!({"v":1,"request_id":"12080000-0000-4000-8000-000000000001","kind":kind,"payload":payload})).unwrap())
}
fn run(action: &str, text: &str) -> Result<String, &'static str> {
    convert(action, &json!({"text":text})).map(|v| v["text"].as_str().unwrap().to_string())
}
#[test]
fn encodes_and_decodes_utf8() {
    assert_eq!(
        run("encode", "a b/é?=&+~-._😀").unwrap(),
        "a%20b%2F%C3%A9%3F%3D%26%2B~-._%F0%9F%98%80"
    );
    assert_eq!(
        run("decode", "a%20b%2f%C3%A9+%F0%9F%98%80").unwrap(),
        "a b/é+😀"
    );
    for s in ["", "plain", "100% sûr\n\t", "日本語"] {
        assert_eq!(run("decode", &run("encode", s).unwrap()).unwrap(), s);
    }
}
#[test]
fn rejects_malformed_input() {
    for bad in ["%", "%4", "%zz", "50%", "%+1", "%é1"] {
        assert_eq!(run("decode", bad), Err("INVALID_PERCENT"), "{bad}");
    }
    assert_eq!(run("decode", "%FF"), Err("INVALID_UTF8"));
    assert_eq!(run("decode", "%C3"), Err("INVALID_UTF8"));
    assert_eq!(run("encode", &"a".repeat(32769)), Err("TEXT_TOO_LARGE"));
    assert_eq!(run("encode", &"é".repeat(16384)).unwrap().len(), 16384 * 6);
    assert_eq!(
        convert("encode", &json!({"text":"a","x":1})),
        Err("INVALID_INPUT")
    );
    assert_eq!(convert("nope", &json!({"text":"a"})), Err("UNKNOWN_ACTION"));
}
#[test]
fn reactive_ui_matches_the_action() {
    let mut tool = UrlEncoder::default();
    assert!(call(&mut tool, "init", json!({})).ok);
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"text","event_type":"input","value":"é &"})
        )
        .ok
    );
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"encode","event_type":"click","value":null})
        )
        .ok
    );
    let tree = Value::Object(call(&mut tool, "render", json!({})).data.unwrap());
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    assert_eq!(tree["children"][3]["props"]["value"], "%C3%A9%20%26");
    assert!(call(&mut tool, "shutdown", json!({})).ok);
    assert!(!call(&mut tool, "render", json!({})).ok);
}
