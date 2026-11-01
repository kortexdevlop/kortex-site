use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::ImageEncoder;
use kortex_sdk::{Dispatcher, Host};
use serde_json::{Value, json};
use std::sync::Mutex;
use tool_compresseur_images::{CHUNK, ImageTool};

static CALLS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());

fn transport(bytes: &[u8]) -> i32 {
    CALLS
        .lock()
        .unwrap()
        .push(serde_json::from_slice(bytes).unwrap());
    0
}

fn call(tool: &mut Dispatcher<ImageTool>, kind: &str, payload: Value) -> Value {
    serde_json::from_slice(
        &tool.dispatch(
            &serde_json::to_vec(&json!({
                "v":1,
                "request_id":"12100000-0000-4000-8000-ffffffffffff",
                "kind":kind,
                "payload":payload
            }))
            .unwrap(),
        ),
    )
    .unwrap()
}

fn click(tool: &mut Dispatcher<ImageTool>, key: &str) {
    assert_eq!(
        call(
            tool,
            "ui.event",
            json!({"node_key":key,"event_type":"click","value":null})
        )["ok"],
        true,
        "{key}"
    );
}

fn next() -> Value {
    CALLS.lock().unwrap().remove(0)
}

fn result(tool: &mut Dispatcher<ImageTool>, request: &Value, data: Value) {
    assert_eq!(
        call(
            tool,
            "capability.result",
            json!({"capability_request_id":request["request_id"],"ok":true,"data":data})
        )["ok"],
        true
    );
}

fn find<'a>(tree: &'a Value, key: &str) -> Option<&'a Value> {
    if tree["key"] == key {
        return Some(tree);
    }
    tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|child| find(child, key))
}

fn render(tool: &mut Dispatcher<ImageTool>) -> Value {
    let tree = call(tool, "render", json!({}))["data"].clone();
    kortex_ui::ValidatedUiTree::validate(tree.clone()).unwrap();
    tree
}

fn image() -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut pixels = Vec::new();
    let mut state = 0x1234_5678u32;
    for _ in 0..80 * 60 * 3 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        pixels.push((state >> 24) as u8);
    }
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&pixels, 80, 60, image::ExtendedColorType::Rgb8)
        .unwrap();
    bytes
}

#[test]
fn picker_compresses_saves_and_reports_local_result() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    CALLS.lock().unwrap().clear();
    let mut tool = Dispatcher::new(ImageTool::with_host(Host::new(transport)));
    assert_eq!(call(&mut tool, "init", json!({}))["ok"], true);
    assert_eq!(
        find(&render(&mut tool), "compress").unwrap()["props"]["disabled"],
        true
    );

    click(&mut tool, "pick");
    let pick = next();
    assert_eq!(pick["capability"], "file.pick");
    let source = image();
    result(
        &mut tool,
        &pick,
        json!({"handle":"a".repeat(64),"name":"photo.png","size":source.len()}),
    );
    let read = next();
    assert_eq!(read["capability"], "file.read");
    assert_eq!(read["payload"]["length"], CHUNK);
    result(
        &mut tool,
        &read,
        json!({"data_base64":STANDARD.encode(&source),"offset":0,"eof":true}),
    );

    let ready = render(&mut tool);
    assert_eq!(
        find(&ready, "compress").unwrap()["props"]["disabled"],
        false
    );
    click(&mut tool, "compress");
    let compressed = render(&mut tool);
    assert_eq!(
        find(&compressed, "save").unwrap()["props"]["disabled"],
        false
    );

    click(&mut tool, "save");
    let save = next();
    assert_eq!(save["capability"], "file.save");
    assert_eq!(save["payload"]["name"], "photo-compresse.jpg");
    let saved = STANDARD
        .decode(save["payload"]["data_base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        image::guess_format(&saved).unwrap(),
        image::ImageFormat::Jpeg
    );
    result(&mut tool, &save, json!({"saved":true}));
    assert!(
        find(&render(&mut tool), "notice").unwrap()["props"]["text"]
            .as_str()
            .unwrap()
            .contains("enregistrée")
    );
}

#[test]
fn quality_selection_before_file_does_not_enable_compression() {
    let _serial = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
    CALLS.lock().unwrap().clear();
    let mut tool = Dispatcher::new(ImageTool::with_host(Host::new(transport)));
    assert_eq!(call(&mut tool, "init", json!({}))["ok"], true);
    assert_eq!(
        call(
            &mut tool,
            "ui.event",
            json!({"node_key":"quality","event_type":"change","value":"60"})
        )["ok"],
        true
    );
    assert_eq!(
        find(&render(&mut tool), "compress").unwrap()["props"]["disabled"],
        true
    );
}
