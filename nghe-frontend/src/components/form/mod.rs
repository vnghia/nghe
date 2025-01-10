mod error;
pub mod input;
mod submit;

use leptos::html;
use leptos::prelude::*;
use web_sys::MouseEvent;

pub fn Form<IV: IntoView, I: Send + Sync + 'static>(
    title: &'static str,
    fields: impl Fn() -> IV,
    button: &'static str,
    on_click: impl Fn(MouseEvent) + 'static,
    action: Action<I, Result<(), String>, SyncStorage>,
) -> impl IntoView {
    html::div()
        .class(
            "w-full bg-white rounded-lg shadow dark:border md:mt-0 sm:max-w-md xl:p-0 \
             dark:bg-gray-800 dark:border-gray-700",
        )
        .child(
            html::div().class("p-6 space-y-4 md:space-y-6 sm:p-8").child((
                html::h1()
                    .class(
                        "text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl \
                         dark:text-white",
                    )
                    .child(title),
                html::div().class("space-y-4 md:space-y-6").child((
                    fields(),
                    submit::Submit(button, on_click, action),
                    error::Error(action),
                )),
            )),
        )
}
