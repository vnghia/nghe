use anyhow::Result;
use dioxus::prelude::*;

use crate::Route;

static ERROR_SIGNAL: GlobalSignal<Option<String>> = Signal::global(Default::default);

struct ErrorToast;

impl ErrorToast {
    fn write(e: &anyhow::Error) {
        *ERROR_SIGNAL.write() = Some(e.root_cause().to_string());
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

impl<T> Toast for Result<T, anyhow::Error> {
    type Out = T;

    fn toast(self) -> Option<T> {
        match self {
            Ok(t) => {
                ErrorToast::clear();
                Some(t)
            }
            Err(e) => {
                ErrorToast::write(&e);
                None
            }
        }
    }
}

impl<T> Toast for Result<T, &anyhow::Error> {
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

impl Toast for anyhow::Error {
    type Out = ();

    fn toast(self) -> Option<()> {
        ErrorToast::write(&self);
        None
    }
}

#[component]
pub fn Error() -> Element {
    rsx! {
        Outlet::<Route> {}
        if let Some(e) = ERROR_SIGNAL.as_ref() {
            div { class: "toast",
                div { class: "alert alert-error", "{e}" }
            }
        }
    }
}
