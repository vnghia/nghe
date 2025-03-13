pub mod error;
pub mod input;
mod submit;

use leptos::prelude::*;
use leptos::{html, svg};
use web_sys::MouseEvent;

use crate::Error;

pub fn Form<IV: IntoView, I: Send + Sync + 'static>(
    title: &'static str,
    modal_id: Option<&'static str>,
    fields: impl Fn() -> IV,
    button: &'static str,
    on_click: impl Fn(MouseEvent) + 'static,
    action: Action<I, Result<(), Error>, SyncStorage>,
) -> impl IntoView {
    html::div()
        .class(
            "w-full bg-white rounded-lg shadow dark:border md:mt-0 sm:max-w-md xl:p-0 \
             dark:bg-gray-800 dark:border-gray-700",
        )
        .child(
            html::div().class("p-6 space-y-4 md:space-y-6 sm:p-8").child((
                html::div().class("flex items-center justify-between").child((
                    html::h3()
                        .class(if modal_id.is_some() {
                            "text-lg font-semibold leading-tight tracking-tight text-gray-900 \
                             md:text-xl dark:text-white"
                        } else {
                            "text-xl font-semibold leading-tight tracking-tight text-gray-900 \
                             md:text-2xl dark:text-white"
                        })
                        .child(title),
                    modal_id.map(|modal_id| {
                        html::button()
                            .r#type("button")
                            .attr("data-modal-hide", modal_id)
                            .class(
                                "bg-transparent hover:bg-gray-200 text-gray-900 rounded-lg \
                                 text-sm w-8 h-8 ms-auto inline-flex justify-center items-center \
                                 dark:hover:bg-gray-600 dark:hover:text-white",
                            )
                            .child((
                                svg::svg()
                                    .aria_hidden("true")
                                    .attr("fill", "none")
                                    .attr("viewBox", "0 0 14 14")
                                    .attr("xmlns", "http://www.w3.org/2000/svg")
                                    .class("w-3 h-3")
                                    .child(
                                        svg::path()
                                            .attr("stroke", "currentColor")
                                            .attr("stroke-linecap", "round")
                                            .attr("stroke-linejoin", "round")
                                            .attr("stroke-width", "2")
                                            .attr("d", "m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"),
                                    ),
                                html::span().class("sr-only").child("Close modal"),
                            ))
                    }),
                )),
                html::div().class("space-y-4 md:space-y-6").child((
                    fields(),
                    submit::Submit(button, on_click, action),
                    error::Error(action),
                )),
            )),
        )
}
