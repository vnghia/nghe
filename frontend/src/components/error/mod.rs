use std::borrow::Cow;

use anyhow::Result;
use dioxus::prelude::*;

use crate::Route;

static ERROR_SIGNAL: GlobalSignal<Option<Cow<'static, str>>> = Signal::global(Default::default);

struct ErrorToast;

impl ErrorToast {
    fn write<E: Into<Cow<'static, str>>>(e: E) {
        *ERROR_SIGNAL.write() = Some(e.into());
    }

    fn clear() {
        *ERROR_SIGNAL.write() = None;
    }
}

// Based on dioxus error boundary
pub trait Toast {
    type Out = ();

    fn toast(self) -> Option<Self::Out>;
}

impl<T> Toast for Result<T, anyhow::Error> {
    type Out = T;

    fn toast(self) -> Option<Self::Out> {
        match self {
            Ok(t) => {
                ErrorToast::clear();
                Some(t)
            }
            Err(e) => {
                ErrorToast::write(e.root_cause().to_string());
                None
            }
        }
    }
}

impl<T> Toast for Result<T, &anyhow::Error> {
    type Out = T;

    fn toast(self) -> Option<Self::Out> {
        match self {
            Ok(t) => {
                ErrorToast::clear();
                Some(t)
            }
            Err(e) => {
                ErrorToast::write(e.root_cause().to_string());
                None
            }
        }
    }
}

impl Toast for &'static str {
    fn toast(self) -> Option<Self::Out> {
        ErrorToast::write(self);
        None
    }
}

impl Toast for String {
    fn toast(self) -> Option<Self::Out> {
        ErrorToast::write(self);
        None
    }
}

#[component]
pub fn Error() -> Element {
    rsx! {
        Outlet::<Route> {}
        if let Some(e) = ERROR_SIGNAL.as_ref() {
            div { class: "toast",
                div {
                    class: "alert alert-error",
                    onclick: |_| ErrorToast::clear(),
                    span { class: "text-error-content", "{e}" }
                }
            }
        }
    }
}
