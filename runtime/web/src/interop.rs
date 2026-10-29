use wasm_bindgen::prelude::*;
#[wasm_bindgen(js_namespace = kortexInterop)]
extern "C" {
    #[wasm_bindgen(js_name=subscribe)]
    pub fn subscribe(callback: &js_sys::Function) -> js_sys::Function;
    #[wasm_bindgen(js_name=command)]
    pub fn command(value: JsValue);
}
