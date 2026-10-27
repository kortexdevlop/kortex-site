use wasm_bindgen::prelude::*;
#[wasm_bindgen(raw_module = "/adapter/interop.js")]
extern "C" {
    #[wasm_bindgen(js_name=subscribe)]
    pub fn subscribe(callback: &js_sys::Function) -> js_sys::Function;
    #[wasm_bindgen(js_name=command)]
    pub fn command(value: JsValue);
}
