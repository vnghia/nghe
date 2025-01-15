use leptos::html::ElementChild;
use leptos::prelude::ClassAttribute;
use leptos::{IntoView, html};

use crate::error;

pub fn Http(error: error::Http) -> impl IntoView {
    html::section().class("h-full w-full bg-gray-50 dark:bg-gray-900").child(
        html::div().class("py-20 px-8 mx-auto max-w-screen-xl lg:py-30 lg:px-12").child(
            html::div().class("mx-auto max-w-screen-sm text-center").child((
                html::h1()
                    .class(
                        "mb-4 text-7xl tracking-tight font-extrabold lg:text-9xl text-red-500 \
                         dark:text-red-400",
                    )
                    .child(error.code),
                html::p()
                    .class("mb-4 text-2xl tracking-tight text-gray-900 md:text-3xl dark:text-white")
                    .child(error.text),
            )),
        ),
    )
}
