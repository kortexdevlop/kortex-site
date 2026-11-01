use kortex_sdk::{Request, Tool};
use serde_json::{Value, json};
use tool_convertisseur_couleurs::{ColorConverter, convert};
fn call(tool: &mut ColorConverter, kind: &str, payload: Value) -> kortex_sdk::Reply {
    tool.handle(serde_json::from_value::<Request>(json!({"v":1,"request_id":"12060000-0000-4000-8000-000000000001","kind":kind,"payload":payload})).unwrap())
}
fn run(action: &str, text: &str) -> Result<String, &'static str> {
    convert(action, &json!({"text":text})).map(|v| v["text"].as_str().unwrap().to_string())
}
#[test]
fn converts_between_all_formats() {
    assert_eq!(run("rgb", "#1f4fa3").unwrap(), "rgb(31, 79, 163)");
    assert_eq!(run("hsl", "#1F4FA3").unwrap(), "hsl(218, 68%, 38%)");
    assert_eq!(run("hex", "rgb(31 79 163)").unwrap(), "#1f4fa3");
    assert_eq!(run("hex", "hsl(0, 100%, 50%)").unwrap(), "#ff0000");
    assert_eq!(run("hex", "hsl(120, 100%, 25%)").unwrap(), "#008000");
    assert_eq!(run("rgb", "#fff").unwrap(), "rgb(255, 255, 255)");
    assert_eq!(run("hsl", " #000 ").unwrap(), "hsl(0, 0%, 0%)");
    assert_eq!(run("hsl", "#ff0000").unwrap(), "hsl(0, 100%, 50%)");
    assert_eq!(run("hsl", "#ff00ff").unwrap(), "hsl(300, 100%, 50%)");
    assert_eq!(run("hex", "hsl(360, 100%, 50%)").unwrap(), "#ff0000");
}
#[test]
fn rejects_invalid_input() {
    for bad in [
        "",
        "#",
        "#12",
        "#12345g",
        "#1234567",
        "rgb(256, 0, 0)",
        "rgb(1, 2)",
        "rgb(-1, 0, 0)",
        "rgb(1.5, 0, 0)",
        "hsl(361, 0%, 0%)",
        "hsl(0, 101%, 0%)",
        "hsl(0, 50, 50)",
        "red",
    ] {
        assert_eq!(run("hex", bad), Err("INVALID_COLOR"), "{bad}");
    }
    assert_eq!(
        convert("hex", &json!({"text":"#fff","x":1})),
        Err("INVALID_INPUT")
    );
    assert_eq!(
        convert("nope", &json!({"text":"#fff"})),
        Err("UNKNOWN_ACTION")
    );
    assert_eq!(run("hex", &"1".repeat(65)), Err("TEXT_TOO_LARGE"));
}
#[test]
fn reactive_ui_matches_the_action() {
    let mut tool = ColorConverter::default();
    assert!(call(&mut tool, "init", json!({})).ok);
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"text","event_type":"input","value":"#1f4fa3"})
        )
        .ok
    );
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"rgb","event_type":"click","value":null})
        )
        .ok
    );
    let tree = Value::Object(call(&mut tool, "render", json!({})).data.unwrap());
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    assert_eq!(tree["children"][3]["props"]["value"], "rgb(31, 79, 163)");
    assert!(call(&mut tool, "shutdown", json!({})).ok);
    assert!(!call(&mut tool, "render", json!({})).ok);
}
