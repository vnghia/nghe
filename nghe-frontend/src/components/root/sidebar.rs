use leptos::prelude::*;
use leptos::{html, svg};
use leptos_router::components::A;
use nghe_api::user::Role;

pub fn Sidebar(user_role: Role) -> impl IntoView {
    html::aside()
        .id("drawer-navigation")
        .aria_label("Sidenav")
        .class(
            "fixed top-0 left-0 z-40 w-64 h-screen pt-14 transition-transform -translate-x-full \
             bg-white border-r border-gray-200 md:translate-x-0 dark:bg-gray-800 \
             dark:border-gray-700",
        )
        .child(
            html::div().class("overflow-y-auto py-5 px-3 h-full bg-white dark:bg-gray-800").child(
                html::ul().class("space-y-2").child(if user_role.admin {
                    Some((html::li().child(
                        A(component_props_builder(&A)
                            .href("users")
                            .children(Box::new(move || {
                                (
                                    svg::svg()
                                        .aria_hidden("true")
                                        .attr("fill", "none")
                                        .attr("viewBox", "0 0 24 24")
                                        .attr("xmlns", "http://www.w3.org/2000/svg")
                                        .class(
                                            "w-7 h-7 text-gray-500 transition duration-75 \
                                             dark:text-gray-400 group-hover:text-gray-900 \
                                             dark:group-hover:text-white",
                                        )
                                        .child(
                                            svg::path()
                                                .attr("stroke", "currentColor")
                                                .attr("stroke-linecap", "round")
                                                .attr("stroke-width", "2")
                                                .attr(
                                                    "d",
                                                    "M4.5 17H4a1 1 0 0 1-1-1 3 3 0 0 1 \
                                                     3-3h1m0-3.05A2.5 2.5 0 1 1 9 5.5M19.5 \
                                                     17h.5a1 1 0 0 0 1-1 3 3 0 0 \
                                                     0-3-3h-1m0-3.05a2.5 2.5 0 1 0-2-4.45m.5 \
                                                     13.5h-7a1 1 0 0 1-1-1 3 3 0 0 1 3-3h3a3 3 0 \
                                                     0 1 3 3 1 1 0 0 1-1 1Zm-1-9.5a2.5 2.5 0 1 \
                                                     1-5 0 2.5 2.5 0 0 1 5 0Z",
                                                ),
                                        ),
                                    html::span().class("ml-3").child("Users"),
                                )
                                    .into_any()
                            }))
                            .build())
                        .attr(
                            "class",
                            "flex items-center p-2 text-base font-medium text-gray-900 rounded-lg \
                             dark:text-white hover:bg-gray-100 dark:hover:bg-gray-700 group",
                        ),
                    ),))
                } else {
                    None
                }),
            ),
        )
}
