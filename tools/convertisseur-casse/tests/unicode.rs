use kortex_sdk::{Request, Tool};
use serde_json::{Value, json};
use tool_convertisseur_casse::{CaseConverter, convert};
fn call(tool: &mut CaseConverter, kind: &str, payload: Value) -> kortex_sdk::Reply {
    tool.handle(serde_json::from_value::<Request>(json!({"v":1,"request_id":"12000000-0000-4000-8000-000000000001","kind":kind,"payload":payload})).unwrap())
}
#[test]
fn unicode_and_documented_french_initials() {
    for (action, input, expected) in [
        ("upper", "été ß 😀", "ÉTÉ SS 😀"),
        ("lower", "ÉTÉ İ 😀", "été i\u{307} 😀"),
        ("title", "l'ÉTÉ - 42éLÈVES", "L'été - 42Élèves"),
        ("title", "e\u{301}COLE", "E\u{301}cole"),
        ("lower", "", ""),
    ] {
        assert_eq!(
            convert(action, &json!({"text":input})).unwrap(),
            json!({"text":expected})
        );
    }
}
#[test]
fn rejects_unknown_fields_and_byte_overflow() {
    assert!(convert("upper", &json!({"text":"a","extra":1})).is_err());
    assert!(convert("upper", &json!({"text":"é".repeat(16385)})).is_err());
    assert_eq!(
        convert("upper", &json!({"text":"a".repeat(32768)})).unwrap()["text"]
            .as_str()
            .unwrap()
            .len(),
        32768
    );
    assert!(convert("unknown", &json!({"text":"a"})).is_err());
}
#[test]
fn reactive_ui_uses_the_identical_action_and_valid_schema() {
    let mut tool = CaseConverter::default();
    assert!(call(&mut tool, "init", json!({})).ok);
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"text","event_type":"input","value":"ÉTÉ ß"})
        )
        .ok
    );
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"upper","event_type":"click","value":null})
        )
        .ok
    );
    let tree = Value::Object(call(&mut tool, "render", json!({})).data.unwrap());
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    assert_eq!(
        tree["children"][3]["props"]["value"],
        convert("upper", &json!({"text":"ÉTÉ ß"})).unwrap()["text"]
    );
    assert!(call(&mut tool, "shutdown", json!({})).ok);
    assert!(!call(&mut tool, "render", json!({})).ok);
}
