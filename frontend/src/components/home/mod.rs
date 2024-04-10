use dioxus::prelude::*;

use crate::state::use_common_state;

#[component]
pub fn Home() -> Element {
    let common_state = use_common_state();
    rsx! {
        div { "Home" }
    }
}
