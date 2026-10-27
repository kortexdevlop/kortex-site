use kortex_sdk::{Dispatcher, Request, RequestKind, Tool};
use serde_json::{Value, json};
use tool_calculatrice::Calculator;
fn request(kind: RequestKind, payload: Value) -> Request {
    Request {
        v: 1,
        request_id: "00000000-0000-4000-8000-000000000001".into(),
        kind,
        payload: payload.as_object().unwrap().clone(),
    }
}
#[test]
fn arithmetic_contract_corpus() {
    let mut tool = Calculator::default();
    tool.handle(request(RequestKind::Init, json!({})));
    let cases: Vec<Value> = serde_json::from_str(include_str!("action_cases.json")).unwrap();
    for case in cases {
        let reply = tool.handle(request(
            RequestKind::ActionInvoke,
            json!({"action_id":case["action"],"input":case["input"]}),
        ));
        if let Some(code) = case["error"].as_str() {
            assert_eq!(reply.error.unwrap().code, code, "{case}");
        } else {
            assert!(reply.ok, "{case}");
            assert_eq!(
                reply.data.unwrap()["result"].as_f64(),
                case["expected"]["result"].as_f64(),
                "{case}"
            );
        }
    }
}
#[test]
fn lifecycle_keypad_and_ui_action_agree() {
    let mut tool = Calculator::default();
    assert!(!tool.handle(request(RequestKind::Render, json!({}))).ok);
    tool.handle(request(RequestKind::Init, json!({})));
    for key in [
        "digit-2", "digit-5", "target-b", "digit-4", "digit-8", "multiply",
    ] {
        assert!(
            tool.handle(request(
                RequestKind::UiEvent,
                json!({"node_key":key,"event_type":"click","value":null})
            ))
            .ok
        );
    }
    let ui = tool
        .handle(request(RequestKind::Render, json!({})))
        .data
        .unwrap();
    assert!(
        serde_json::to_string(&ui)
            .unwrap()
            .contains("Résultat : 1200")
    );
    assert_eq!(
        tool.handle(request(
            RequestKind::ActionInvoke,
            json!({"action_id":"multiply","input":{"a":25,"b":48}})
        ))
        .data
        .unwrap()["result"],
        1200.0
    );
    for key in ["clear", "sign", "decimal", "digit-5", "backspace"] {
        assert!(
            tool.handle(request(
                RequestKind::UiEvent,
                json!({"node_key":key,"event_type":"click","value":null})
            ))
            .ok
        );
    }
    tool.handle(request(RequestKind::Shutdown, json!({})));
    assert!(!tool.handle(request(RequestKind::Render, json!({}))).ok);
    tool.handle(request(RequestKind::Init, json!({})));
    assert!(
        !serde_json::to_string(&tool.handle(request(RequestKind::Render, json!({}))).data)
            .unwrap()
            .contains("1200")
    );
}
#[test]
fn frozen_wire_goldens() {
    let mut dispatcher = Dispatcher::new(Calculator::default());
    for line in include_str!("../../../tests/golden/calculatrice.ndjson").lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let actual: Value = serde_json::from_slice(
            &dispatcher.dispatch(&serde_json::to_vec(&case["request"]).unwrap()),
        )
        .unwrap();
        assert_eq!(actual, case["reply"]);
    }
}
