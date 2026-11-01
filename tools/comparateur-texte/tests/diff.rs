use kortex_sdk::{Request, Tool};
use serde_json::{Value, json};
use tool_comparateur_texte::{TextComparer, diff};
fn call(tool: &mut TextComparer, kind: &str, payload: Value) -> kortex_sdk::Reply {
    tool.handle(serde_json::from_value::<Request>(json!({"v":1,"request_id":"12090000-0000-4000-8000-000000000001","kind":kind,"payload":payload})).unwrap())
}
#[test]
fn reports_added_and_removed_lines() {
    let out = diff(&json!({"a":"un\ndeux\ntrois\n","b":"un\n2\ntrois\nquatre"})).unwrap();
    assert_eq!(out["text"], "  un\n- deux\n+ 2\n  trois\n+ quatre");
    assert_eq!(
        (out["added"].clone(), out["removed"].clone()),
        (json!(2), json!(1))
    );
    let same = diff(&json!({"a":"x\r\ny","b":"x\ny"})).unwrap();
    assert_eq!(same["text"], "  x\n  y");
    assert_eq!(
        (same["added"].clone(), same["removed"].clone()),
        (json!(0), json!(0))
    );
    assert_eq!(diff(&json!({"a":"","b":""})).unwrap()["text"], "");
    assert_eq!(diff(&json!({"a":"","b":"n"})).unwrap()["text"], "+ n");
    assert_eq!(diff(&json!({"a":"o","b":""})).unwrap()["text"], "- o");
    assert_eq!(
        diff(&json!({"a":"é\nb","b":"é \nb"})).unwrap()["text"],
        "- é\n+ é \n  b"
    );
}
#[test]
fn enforces_bounds_and_shape() {
    let lines = |n: usize| "x\n".repeat(n);
    assert!(diff(&json!({"a":lines(400),"b":lines(400)})).is_ok());
    assert_eq!(diff(&json!({"a":lines(401),"b":""})), Err("TOO_MANY_LINES"));
    assert_eq!(
        diff(&json!({"a":"a".repeat(16385),"b":""})),
        Err("TEXT_TOO_LARGE")
    );
    assert_eq!(diff(&json!({"a":"a"})), Err("INVALID_INPUT"));
    assert_eq!(diff(&json!({"a":"a","b":1})), Err("INVALID_INPUT"));
    assert_eq!(
        diff(&json!({"a":"a","b":"b","c":"c"})),
        Err("INVALID_INPUT")
    );
    let worst = diff(&json!({"a":"y\n".repeat(400),"b":"z\n".repeat(400)})).unwrap();
    assert!(worst["text"].as_str().unwrap().len() <= 40000);
}
#[test]
fn reactive_ui_matches_the_action() {
    let mut tool = TextComparer::default();
    assert!(call(&mut tool, "init", json!({})).ok);
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"a","event_type":"input","value":"a\nb"})
        )
        .ok
    );
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"b","event_type":"input","value":"a\nc"})
        )
        .ok
    );
    assert!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"compare","event_type":"click","value":null})
        )
        .ok
    );
    let tree = Value::Object(call(&mut tool, "render", json!({})).data.unwrap());
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    assert_eq!(tree["children"][4]["props"]["value"], "  a\n- b\n+ c");
    assert_eq!(
        tree["children"][5]["props"]["text"],
        "1 ligne(s) ajoutée(s), 1 supprimée(s)."
    );
    let reply = call(
        &mut tool,
        "action.invoke",
        json!({"action_id":"diff","input":{"a":"a","b":"a"}}),
    );
    assert_eq!(reply.data.unwrap()["added"], 0);
    assert!(call(&mut tool, "shutdown", json!({})).ok);
    assert!(!call(&mut tool, "render", json!({})).ok);
}
