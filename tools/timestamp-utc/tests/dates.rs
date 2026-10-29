use kortex_sdk::{Request, Tool};
use serde_json::{Value, json};
use tool_timestamp_utc::{TimestampUtc, convert};
#[test]
fn epoch_negatives_and_roundtrips() {
    for (iso, millis, seconds) in [
        ("1970-01-01T00:00:00Z", 0i64, 0i64),
        ("1969-12-31T23:59:59.999Z", -1, -1),
        ("2000-02-29T12:34:56.123Z", 951827696123, 951827696),
        ("0001-01-01T00:00:00Z", -62135596800000, -62135596800),
        ("9999-12-31T23:59:59.999Z", 253402300799999, 253402300799),
    ] {
        let out = convert("to_unix", &json!({"iso_utc":iso})).unwrap();
        assert_eq!(
            out,
            json!({"iso_utc":iso,"unix_millis":millis,"unix_seconds":seconds})
        );
        assert_eq!(
            convert("from_unix", &json!({"value":millis,"unit":"milliseconds"})).unwrap(),
            out
        );
    }
}
#[test]
fn rejects_ambiguous_invalid_and_submillisecond_dates() {
    for date in [
        "2026-10-27T12:00:00",
        "2026-10-27T12:00:00+01:00",
        "2025-02-29T00:00:00Z",
        "2024-01-01T00:00:00.000001Z",
        "2016-12-31T23:59:60Z",
        "0000-01-01T00:00:00Z",
    ] {
        assert!(
            convert("to_unix", &json!({"iso_utc":date})).is_err(),
            "{date}"
        );
    }
}
#[test]
fn explicit_units_integer_and_overflow() {
    assert_eq!(
        convert("from_unix", &json!({"value":-1,"unit":"seconds"})).unwrap()["unix_millis"],
        -1000
    );
    for input in [
        json!({"value":1}),
        json!({"value":1.1,"unit":"seconds"}),
        json!({"value":i64::MAX,"unit":"seconds"}),
        json!({"value":253402300800000i64,"unit":"milliseconds"}),
        json!({"value":0,"unit":"milliseconds","extra":true}),
    ] {
        assert!(convert("from_unix", &input).is_err());
    }
}
#[test]
fn reactive_ui_and_autonomous_action_agree() {
    let mut tool = TimestampUtc::default();
    let mut call = |kind: &str, payload: Value| {
        tool.handle(serde_json::from_value::<Request>(json!({"v":1,"request_id":"12020000-0000-4000-8000-000000000001","kind":kind,"payload":payload})).unwrap())
    };
    assert!(call("init", json!({})).ok);
    assert!(
        call(
            "ui.event",
            json!({"node_key":"unix","event_type":"input","value":"-1"})
        )
        .ok
    );
    let tree = Value::Object(call("render", json!({})).data.unwrap());
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    assert!(tree.to_string().contains("1969-12-31T23:59:59.999Z"));
    let out = call(
        "action.invoke",
        json!({"action_id":"from_unix","input":{"value":-1,"unit":"milliseconds"}}),
    );
    assert!(out.ok);
    assert_eq!(out.data.unwrap()["unix_millis"], -1);
}
