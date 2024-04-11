use dioxus::prelude::*;

use crate::Route;

static ERROR_SIGNAL: GlobalSignal<Option<anyhow::Error>> = Signal::global(Default::default);

struct ErrorToast;

impl ErrorToast {
    fn write<E: Into<anyhow::Error>>(e: E) {
        *ERROR_SIGNAL.write() = Some(e.into());
    }

    fn clear() {
        *ERROR_SIGNAL.write() = None;
    }
}

// Based on dioxus error boundary
pub trait Toast {
    type Out;

    fn toast(self) -> Option<Self::Out>;
}

impl<T, E: Into<anyhow::Error>> Toast for Result<T, E> {
    type Out = T;

    fn toast(self) -> Option<T> {
        match self {
            Ok(t) => {
                ErrorToast::clear();
                Some(t)
            }
            Err(e) => {
                ErrorToast::write(e);
                None
            }
        }
    }
}

#[component]
pub fn Error() -> Element {
    rsx! {
        Outlet::<Route> {}
        if let Some(e) = ERROR_SIGNAL.as_ref() {
            div { class: "toast",
                div { class: "alert alert-error", "{e.root_cause()}" }
            }
        }
    }
}
