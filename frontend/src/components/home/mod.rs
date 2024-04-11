use dioxus::prelude::*;

use crate::state::CommonState;

#[component]
pub fn Home() -> Element {
    let _common_state = CommonState::use_redirect();
    rsx! {
        div { "Home" }
    }
}
