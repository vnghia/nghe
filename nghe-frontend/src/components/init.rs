use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "export function initializeFlowbite() { initFlowbite(); }")]
extern "C" {
    fn initializeFlowbite();
}

pub fn flowbite() {
    initializeFlowbite();
    leptos::logging::debug_warn!("initializeFlowbite called");
}

pub fn flowbite_suspense<T>() -> NodeRef<T>
where
    T: html::ElementType + 'static,
    T::Output: Clone + wasm_bindgen::JsCast,
{
    let node_ref = NodeRef::<T>::new();
    Effect::new(move || {
        node_ref.track();
        flowbite();
    });
    node_ref
}
