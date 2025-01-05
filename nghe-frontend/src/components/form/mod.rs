pub mod input;

use leptos::html;
use leptos::prelude::*;

pub fn Form<IV: IntoView>(
    title: &'static str,
    fields: impl Fn() -> IV,
    button: &'static str,
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
                    html::button()
                        .class(
                            "w-full text-white bg-primary-600 hover:bg-primary-700 focus:ring-4 \
                             focus:outline-none focus:ring-primary-300 font-medium rounded-lg \
                             text-sm px-5 py-2.5 text-center dark:bg-primary-600 \
                             dark:hover:bg-primary-700 dark:focus:ring-primary-800",
                        )
                        .child(button),
                )),
            )),
        )
}
