use base64::{Engine as _, engine::general_purpose::STANDARD};
use kortex_sdk::{Dispatcher, Host};
use serde_json::{Value, json};
use std::sync::Mutex;
use tool_fusion_pdf::{CHUNK, FILE_LIMIT, PdfTool, pdf};

static CALLS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
fn transport(bytes: &[u8]) -> i32 {
    CALLS
        .lock()
        .unwrap()
        .push(serde_json::from_slice(bytes).unwrap());
    0
}
fn call(tool: &mut Dispatcher<PdfTool>, kind: &str, payload: Value) -> Value {
    serde_json::from_slice(&tool.dispatch(&serde_json::to_vec(&json!({"v":1,"request_id":"12040000-0000-4000-8000-ffffffffffff","kind":kind,"payload":payload})).unwrap())).unwrap()
}
fn click(tool: &mut Dispatcher<PdfTool>, key: &str) {
    let reply = call(
        tool,
        "ui.event",
        json!({"node_key":key,"event_type":"click","value":null}),
    );
    assert_eq!(reply["ok"], true, "{key}");
}
fn next() -> Value {
    CALLS.lock().unwrap().remove(0)
}
fn result(tool: &mut Dispatcher<PdfTool>, request: &Value, data: Value) {
    assert_eq!(
        call(
            tool,
            "capability.result",
            json!({"capability_request_id":request["request_id"],"ok":true,"data":data})
        )["ok"],
        true
    );
}
fn render(tool: &mut Dispatcher<PdfTool>) -> Value {
    let tree = call(tool, "render", json!({}))["data"].clone();
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    tree
}
fn fresh() -> Dispatcher<PdfTool> {
    CALLS.lock().unwrap().clear();
    let mut tool = Dispatcher::new(PdfTool::with_host(Host::new(transport)));
    assert_eq!(call(&mut tool, "init", json!({}))["ok"], true);
    tool
}
fn find<'a>(tree: &'a Value, key: &str) -> Option<&'a Value> {
    if tree["key"] == key {
        return Some(tree);
    }
    tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|c| find(c, key))
}
fn status(tree: &Value) -> String {
    find(tree, "status").unwrap()["props"]["text"]
        .as_str()
        .unwrap()
        .into()
}
/// One page whose content is `text`; `padding` bytes of trailing junk force several read chunks.
fn one_page(text: &str, padding: usize) -> Vec<u8> {
    let stream = format!("BT ({text}) Tj ET");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 /MediaBox [0 0 100 100] >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /Contents 4 0 R >>".to_string(),
        format!("<< /Length {} >>\nstream\n{stream}\nendstream", stream.len()),
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n");
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    out.extend(std::iter::repeat_n(b' ', padding));
    out
}
fn load(tool: &mut Dispatcher<PdfTool>, button: &str, name: &str, bytes: &[u8]) {
    click(tool, button);
    let pick = next();
    assert_eq!(pick["capability"], "file.pick");
    result(
        tool,
        &pick,
        json!({"handle":"a".repeat(64),"name":name,"size":bytes.len()}),
    );
    let mut offset = 0;
    while offset < bytes.len() {
        let read = next();
        assert_eq!(read["capability"], "file.read");
        assert_eq!(read["payload"]["offset"], offset);
        let end = (offset + CHUNK).min(bytes.len());
        result(
            tool,
            &read,
            json!({"data_base64":STANDARD.encode(&bytes[offset..end]),"offset":offset,"eof":end==bytes.len()}),
        );
        offset = end;
    }
}

#[test]
fn picks_two_files_merges_swaps_and_saves() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut tool = fresh();
    let first = one_page("first", CHUNK + 1000);
    let second = one_page("second", 0);
    let tree = render(&mut tool);
    assert_eq!(find(&tree, "merge").unwrap()["props"]["disabled"], true);

    load(&mut tool, "pick_first", "a.pdf", &first);
    let tree = render(&mut tool);
    assert_eq!(find(&tree, "merge").unwrap()["props"]["disabled"], true);
    load(&mut tool, "pick_second", "b.pdf", &second);
    let tree = render(&mut tool);
    let rows = find(&tree, "files").unwrap()["props"]["rows"].as_array().unwrap().clone();
    assert_eq!(rows[0]["name"], "a.pdf");
    assert_eq!(rows[1]["name"], "b.pdf");
    assert_eq!(rows[0]["pages"], 1);
    assert_eq!(find(&tree, "merge").unwrap()["props"]["disabled"], false);
    assert_eq!(find(&tree, "save").unwrap()["props"]["disabled"], true);

    click(&mut tool, "merge");
    let tree = render(&mut tool);
    assert!(status(&tree).contains("2 pages"), "{}", status(&tree));
    assert_eq!(find(&tree, "save").unwrap()["props"]["disabled"], false);

    click(&mut tool, "swap");
    let tree = render(&mut tool);
    let rows = find(&tree, "files").unwrap()["props"]["rows"].as_array().unwrap().clone();
    assert_eq!(rows[0]["name"], "b.pdf");
    assert_eq!(find(&tree, "save").unwrap()["props"]["disabled"], true, "swap invalidates the result");

    click(&mut tool, "merge");
    click(&mut tool, "save");
    let save = next();
    assert_eq!(save["capability"], "file.save");
    assert_eq!(save["payload"]["name"], "fusion.pdf");
    let saved = STANDARD.decode(save["payload"]["data_base64"].as_str().unwrap()).unwrap();
    assert_eq!(pdf::page_count(&saved), Ok(2));
    let text = String::from_utf8_lossy(&saved);
    assert!(
        text.find("(second)").unwrap() < text.find("(first)").unwrap(),
        "after the swap the second file's page comes first"
    );
    result(&mut tool, &save, json!({"saved":true}));
    let tree = render(&mut tool);
    assert!(
        find(&tree, "notice").unwrap()["props"]["text"]
            .as_str()
            .unwrap()
            .contains("enregistré")
    );
}

#[test]
fn refuses_bad_files_and_keeps_the_other_slot() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut tool = fresh();
    load(&mut tool, "pick_first", "a.pdf", &one_page("first", 0));
    load(&mut tool, "pick_second", "bad.pdf", b"this is plainly not a PDF document");
    let tree = render(&mut tool);
    assert!(status(&tree).contains("INVALID_PDF"));
    let rows = find(&tree, "files").unwrap()["props"]["rows"].as_array().unwrap().clone();
    assert_eq!(rows[0]["name"], "a.pdf");
    assert_eq!(rows[1]["name"], "Aucun fichier");
    assert_eq!(find(&tree, "merge").unwrap()["props"]["disabled"], true);

    click(&mut tool, "pick_second");
    let pick = next();
    result(
        &mut tool,
        &pick,
        json!({"handle":"b".repeat(64),"name":"big.pdf","size":FILE_LIMIT + 1}),
    );
    assert!(status(&render(&mut tool)).contains("TOO_LARGE"));
    assert!(CALLS.lock().unwrap().is_empty(), "no read after a refused pick");
}

#[test]
fn cancelling_a_pick_keeps_loaded_files_and_ignores_late_results() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut tool = fresh();
    load(&mut tool, "pick_first", "a.pdf", &one_page("first", 0));
    click(&mut tool, "pick_second");
    let pick = next();
    click(&mut tool, "cancel");
    result(&mut tool, &pick, json!({"handle":"c".repeat(64),"name":"late.pdf","size":10}));
    assert!(CALLS.lock().unwrap().is_empty(), "stale pick must not start a read");
    let tree = render(&mut tool);
    let rows = find(&tree, "files").unwrap()["props"]["rows"].as_array().unwrap().clone();
    assert_eq!(rows[0]["name"], "a.pdf");
    assert_eq!(rows[1]["name"], "Aucun fichier");

    click(&mut tool, "reset");
    let rows = find(&render(&mut tool), "files").unwrap()["props"]["rows"].as_array().unwrap().clone();
    assert_eq!(rows[0]["name"], "Aucun fichier");
}

#[test]
fn merge_and_save_need_their_prerequisites() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut tool = fresh();
    click(&mut tool, "merge");
    assert!(
        find(&render(&mut tool), "notice").unwrap()["props"]["text"]
            .as_str()
            .unwrap()
            .contains("Choisissez d’abord")
    );
    click(&mut tool, "save");
    assert!(CALLS.lock().unwrap().is_empty());
    assert_eq!(
        call(&mut tool, "ui.event", json!({"node_key":"nope","event_type":"click","value":null}))["ok"],
        false
    );
}

#[test]
fn headless_action_matches_the_ui_merge() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut tool = fresh();
    let reply = call(
        &mut tool,
        "action.invoke",
        json!({"action_id":"merge_pdfs","input":{
            "first_base64":STANDARD.encode(one_page("first", 0)),
            "second_base64":STANDARD.encode(one_page("second", 0)),
        }}),
    );
    assert_eq!(reply["ok"], true);
    assert_eq!(reply["data"]["pages"], 2);
    let reply = call(&mut tool, "action.invoke", json!({"action_id":"nope","input":{}}));
    assert_eq!(reply["ok"], false);
}
