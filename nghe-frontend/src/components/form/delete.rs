use leptos::prelude::*;
use leptos::{ev, html, svg};
use web_sys::MouseEvent;

use super::error;
use crate::Error;

pub fn Delete<I: Send + Sync + 'static>(
    title: &'static str,
    modal_id: &'static str,
    on_click: impl Fn(MouseEvent) + 'static,
    action: Action<I, Result<(), Error>, SyncStorage>,
) -> impl IntoView {
    html::div().class("relative bg-white rounded-lg shadow-sm dark:bg-gray-700").child(
        html::div().class("p-4 md:p-5 text-center space-y-3 md:space-y-4").child((
            svg::svg()
                .aria_hidden("true")
                .attr("fill", "none")
                .attr("viewBox", "0 0 20 20")
                .attr("xmlns", "http://www.w3.org/2000/svg")
                .class("mx-auto w-10 h-10 text-gray-500 dark:text-gray-400")
                .child(
                    svg::path()
                        .attr("stroke", "currentColor")
                        .attr("stroke-linecap", "round")
                        .attr("stroke-linejoin", "round")
                        .attr("stroke-width", "2")
                        .attr("d", "M10 11V6m0 8h.01M19 10a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"),
                ),
            html::h3().class("font-normal text-gray-900 dark:text-white").child(title),
            html::div().child((
                html::button()
                    .r#type("button")
                    .class(
                        "text-white bg-red-600 hover:bg-red-800 focus:ring-4 focus:outline-none \
                         focus:ring-red-300 dark:focus:ring-red-800 font-medium rounded-lg \
                         text-sm inline-flex items-center px-5 py-2.5 text-center",
                    )
                    .child("Yes, I'm sure")
                    .on(ev::click, on_click),
                html::button()
                    .r#type("button")
                    .attr("data-modal-hide", modal_id)
                    .class(
                        "py-2.5 px-5 ms-3 text-sm font-medium text-gray-900 focus:outline-none \
                         bg-white rounded-lg border border-gray-200 hover:bg-gray-100 \
                         hover:text-blue-700 focus:z-10 focus:ring-4 focus:ring-gray-100 \
                         dark:focus:ring-gray-700 dark:bg-gray-800 dark:text-gray-400 \
                         dark:border-gray-600 dark:hover:text-white dark:hover:bg-gray-700",
                    )
                    .child("No, cancel"),
            )),
            error::Error(action, false),
        )),
    )
}
