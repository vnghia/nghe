use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "export function initializeFlowbite() { initFlowbite(); }")]
extern "C" {
    fn initializeFlowbite();
}

pub fn flowbite() {
    initializeFlowbite();
    leptos::logging::debug_warn!("initializeFlowbite called");
}
