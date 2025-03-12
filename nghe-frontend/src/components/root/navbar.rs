use leptos::prelude::*;
use leptos::{html, svg};
use leptos_router::components::A;
use nghe_api::user::get::Response;

pub fn Navbar(user: Response) -> impl IntoView {
    html::nav()
        .class(
            "bg-white border-b border-gray-200 px-4 py-2.5 dark:bg-gray-800 dark:border-gray-700 \
             fixed left-0 right-0 top-0 z-50",
        )
        .child(
            html::div().class("flex flex-wrap justify-between items-center").child((
                html::div().class("flex justify-start items-center").child((
                    html::button()
                        .attr("data-drawer-target", "drawer-navigation")
                        .attr("data-drawer-toggle", "drawer-navigation")
                        .aria_controls("drawer-navigation")
                        .class(
                            "p-2 mr-2 text-gray-600 rounded-lg cursor-pointer md:hidden \
                             hover:text-gray-900 hover:bg-gray-100 focus:bg-gray-100 \
                             dark:focus:bg-gray-700 focus:ring-2 focus:ring-gray-100 \
                             dark:focus:ring-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 \
                             dark:hover:text-white",
                        )
                        .child((
                            svg::svg()
                                .aria_hidden("true")
                                .class("w-6 h-6")
                                .attr("fill", "currentColor")
                                .attr("viewBox", "0 0 20 20")
                                .attr("xmlns", "http://www.w3.org/2000/svg")
                                .child(
                                    svg::path()
                                        .attr("fill-rule", "evenodd")
                                        .attr("clip-rule", "evenodd")
                                        .attr(
                                            "d",
                                            "M3 5a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zM3 \
                                             10a1 1 0 011-1h6a1 1 0 110 2H4a1 1 0 01-1-1zM3 15a1 \
                                             1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z",
                                        ),
                                ),
                            svg::svg()
                                .aria_hidden("true")
                                .class("hidden w-6 h-6")
                                .attr("fill", "currentColor")
                                .attr("viewBox", "0 0 20 20")
                                .attr("xmlns", "http://www.w3.org/2000/svg")
                                .child(
                                    svg::path()
                                        .attr("fill-rule", "evenodd")
                                        .attr("clip-rule", "evenodd")
                                        .attr(
                                            "d",
                                            "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 \
                                             1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 \
                                             01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 \
                                             01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z",
                                        ),
                                ),
                            html::span().class("sr-only").child("Toggle sidebar"),
                        )),
                    A(component_props_builder(&A)
                        .href("")
                        .children(Box::new(move || {
                            html::span()
                                .class(
                                    "self-center text-2xl font-semibold whitespace-nowrap \
                                     dark:text-white",
                                )
                                .child("Nghe")
                                .into_any()
                        }))
                        .build())
                    .attr("class", "flex items-center justify-between mr-4"),
                )),
                html::div().class("flex items-center lg:order-2").child((
                    html::button()
                        .r#type("button")
                        .aria_controls("drawer-navigation")
                        .attr("data-drawer-toggle", "drawer-navigation")
                        .class(
                            "p-2 mr-1 text-gray-500 rounded-lg md:hidden hover:text-gray-900 \
                             hover:bg-gray-100 dark:text-gray-400 dark:hover:text-white \
                             dark:hover:bg-gray-700 focus:ring-4 focus:ring-gray-300 \
                             dark:focus:ring-gray-600",
                        )
                        .child((
                            svg::svg()
                                .aria_hidden("true")
                                .class("w-6 h-6")
                                .attr("fill", "currentColor")
                                .attr("viewBox", "0 0 20 20")
                                .attr("xmlns", "http://www.w3.org/2000/svg")
                                .child(
                                    svg::path()
                                        .attr("fill-rule", "evenodd")
                                        .attr("clip-rule", "evenodd")
                                        .attr(
                                            "d",
                                            "M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 \
                                             3.476l4.817 4.817a1 1 0 01-1.414 \
                                             1.414l-4.816-4.816A6 6 0 012 8z",
                                        ),
                                ),
                            html::span().class("sr-only").child("Toggle search"),
                        )),
                    html::button()
                        .id("user-menu-button")
                        .r#type("button")
                        .aria_expanded("false")
                        .attr("data-dropdown-toggle", "dropdown")
                        .class(
                            "flex mx-3 text-sm rounded-full md:mr-0 focus:ring-4 \
                             focus:ring-gray-300 dark:focus:ring-gray-600",
                        )
                        .child((
                            svg::svg()
                                .aria_hidden("true")
                                .class("w-8 h-8 text-gray-400 rounded-full")
                                .attr("fill", "currentColor")
                                .attr("viewBox", "0 0 24 24")
                                .attr("xmlns", "http://www.w3.org/2000/svg")
                                .child(
                                    svg::path()
                                        .attr("fill-rule", "evenodd")
                                        .attr("clip-rule", "evenodd")
                                        .attr(
                                            "d",
                                            "M12 20a7.966 7.966 0 0 \
                                             1-5.002-1.756l.002.001v-.683c0-1.794 1.492-3.25 \
                                             3.333-3.25h3.334c1.84 0 3.333 1.456 3.333 \
                                             3.25v.683A7.966 7.966 0 0 1 12 20ZM2 12C2 6.477 \
                                             6.477 2 12 2s10 4.477 10 10c0 5.5-4.44 9.963-9.932 \
                                             10h-.138C6.438 21.962 2 17.5 2 12Zm10-5c-1.84 \
                                             0-3.333 1.455-3.333 3.25S10.159 13.5 12 13.5c1.84 0 \
                                             3.333-1.455 3.333-3.25S13.841 7 12 7Z",
                                        ),
                                ),
                            html::span().class("sr-only").child("Open user menu"),
                        )),
                    html::div()
                        .id("dropdown")
                        .class(
                            "hidden z-50 my-4 w-56 text-base list-none bg-white rounded divide-y \
                             divide-gray-100 shadow dark:bg-gray-700 dark:divide-gray-600 \
                             rounded-xl",
                        )
                        .child((
                            html::div().class("py-3 px-4").child((
                                html::span()
                                    .class(
                                        "block text-sm font-semibold text-gray-900 dark:text-white",
                                    )
                                    .child(user.username),
                                html::span()
                                    .class("block text-sm text-gray-900 truncate dark:text-white")
                                    .child(user.email),
                            )),
                            html::ul()
                                .aria_labelledby("dropdown")
                                .class("py-1 text-gray-700 dark:text-gray-300")
                                .child(
                                    html::li().child(
                                        html::span()
                                            .class(
                                                "block py-2 px-4 text-sm hover:bg-gray-100 \
                                                 dark:hover:bg-gray-600 dark:hover:text-white",
                                            )
                                            .child("Sign out"),
                                    ),
                                ),
                        )),
                )),
            )),
        )
}
