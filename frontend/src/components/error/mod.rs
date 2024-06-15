use std::borrow::Cow;

use anyhow::Result;
use dioxus::prelude::*;

static ERROR_SIGNAL: GlobalSignal<Option<Cow<'static, str>>> = Signal::global(Default::default);

pub struct ErrorToast;

impl ErrorToast {
    pub fn write(e: impl Into<Cow<'static, str>>) {
        *ERROR_SIGNAL.write() = Some(e.into());
    }

    pub fn clear() {
        *ERROR_SIGNAL.write() = None;
    }

    pub fn read() -> &'static GlobalSignal<Option<Cow<'static, str>>> {
        &ERROR_SIGNAL
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
