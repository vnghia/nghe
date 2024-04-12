include!(concat!(env!("OUT_DIR"), "/daisy-themes.rs"));

use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use strum::IntoEnumIterator;

use super::error::ErrorToast;
use crate::Route;

impl DaisyTheme {
    const GLOBAL_THEME_KEY: &'static str = "global-theme";

    pub fn use_theme() -> Signal<Self> {
        use_synced_storage::<LocalStorage, Self>(Self::GLOBAL_THEME_KEY.into(), || Self::Light)
    }
}

#[component]
pub fn Global() -> Element {
    let global_theme = DaisyTheme::use_theme()();

    rsx! {
        Outlet::<Route> {}
        for theme in DaisyTheme::iter() {
            input {
                key: "{theme.as_ref()}",
                class: "checkbox theme-controller",
                r#type: "checkbox",
                value: "{theme.as_ref()}",
                checked: theme == global_theme,
                hidden: true
            }
        }
        if let Some(e) = ErrorToast::read().as_ref() {
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
