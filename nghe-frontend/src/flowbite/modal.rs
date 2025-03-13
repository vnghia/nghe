use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "export function showModal(modal_id) { \
                            FlowbiteInstances.getInstance('Modal', modal_id).show(); }")]
extern "C" {
    fn showModal(modal_id: &str);
}

#[wasm_bindgen(inline_js = "export function hideModal(modal_id) { \
                            FlowbiteInstances.getInstance('Modal', modal_id).hide(); }")]
extern "C" {
    fn hideModal(modal_id: &str);
}

pub fn show(modal_id: &'static str) {
    showModal(modal_id);
}

pub fn hide(modal_id: &'static str) {
    hideModal(modal_id);
}
