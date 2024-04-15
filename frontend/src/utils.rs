use anyhow::Result;
use gloo::utils::document;
use wasm_bindgen::JsCast;
use web_sys::HtmlDialogElement;

pub fn show_modal(id: &str) -> Result<()> {
    document()
        .get_element_by_id(id)
        .ok_or(anyhow::anyhow!("element #{} not found", id))?
        .dyn_ref::<HtmlDialogElement>()
        .ok_or(anyhow::anyhow!("element #{} is not a dialog", id))?
        .show_modal()
        .map_err(|_| anyhow::anyhow!("can not show modal #{}", id))
}
