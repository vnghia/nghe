use dioxus::prelude::*;

use crate::Route;

pub static ERROR_SIGNAL: GlobalSignal<Option<anyhow::Error>> = Signal::global(Default::default);

#[component]
pub fn Error() -> Element {
    if let Some(e) = ERROR_SIGNAL.as_ref() {
        let e = e.root_cause();
        rsx! {
            Outlet::<Route> {}
            div { class: "toast",
                div { class: "alert alert-error", "{e}" }
            }
        }
    } else {
        rsx! {
            Outlet::<Route> {}
        }
    }
}
