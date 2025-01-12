use leptos::prelude::*;
use leptos::{html, svg};
use nghe_api::user::info::Request;

use crate::client::Client;

pub fn Shell<IV: IntoView + 'static>(
    child: impl Fn() -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let (client, _) = Client::use_client_redirect();
    let user_info = LocalResource::new(move || async move {
        let client = client().expect(Client::EXPECT_MSG);
        client.json(&Request).await.unwrap()
    });

    Suspense(
        component_props_builder(&Suspense)
            .fallback(move || "Loading")
            .children(ToChildren::to_children(move || {
                Suspend::new(async move {
                    let user_info = user_info.await;
                    html::div().class("antialiased bg-gray-50 dark:bg-gray-900").child((
                        html::nav()
                            .class(
                                "bg-white border-b border-gray-200 px-4 py-2.5 dark:bg-gray-800 \
                                 dark:border-gray-700 fixed left-0 right-0 top-0 z-50",
                            )
                            .child(
                                html::div()
                                    .class("flex flex-wrap justify-between items-center")
                                    .child((
                                        html::div().class("flex justify-start items-center").child(
                                            (
                                                html::button()
                                                    .attr("data-drawer-target", "drawer-navigation")
                                                    .attr("data-drawer-toggle", "drawer-navigation")
                                                    .aria_controls("drawer-navigation")
                                                    .class(
                                                        "p-2 mr-2 text-gray-600 rounded-lg \
                                                         cursor-pointer md:hidden \
                                                         hover:text-gray-900 hover:bg-gray-100 \
                                                         focus:bg-gray-100 dark:focus:bg-gray-700 \
                                                         focus:ring-2 focus:ring-gray-100 \
                                                         dark:focus:ring-gray-700 \
                                                         dark:text-gray-400 \
                                                         dark:hover:bg-gray-700 \
                                                         dark:hover:text-white",
                                                    )
                                                    .child((
                                                        svg::svg()
                                                            .aria_hidden("true")
                                                            .class("w-6 h-6")
                                                            .attr("fill", "currentColor")
                                                            .attr("viewBox", "0 0 20 20")
                                                            .attr(
                                                                "xmlns",
                                                                "http://www.w3.org/2000/svg",
                                                            )
                                                            .child(
                                                                svg::path()
                                                                    .attr("fill-rule", "evenodd")
                                                                    .attr("clip-rule", "evenodd")
                                                                    .attr(
                                                                        "d",
                                                                        "M3 5a1 1 0 011-1h12a1 1 \
                                                                         0 110 2H4a1 1 0 \
                                                                         01-1-1zM3 10a1 1 0 \
                                                                         011-1h6a1 1 0 110 2H4a1 \
                                                                         1 0 01-1-1zM3 15a1 1 0 \
                                                                         011-1h12a1 1 0 110 2H4a1 \
                                                                         1 0 01-1-1z",
                                                                    ),
                                                            ),
                                                        svg::svg()
                                                            .aria_hidden("true")
                                                            .class("w-6 h-6")
                                                            .attr("fill", "currentColor")
                                                            .attr("viewBox", "0 0 20 20")
                                                            .attr(
                                                                "xmlns",
                                                                "http://www.w3.org/2000/svg",
                                                            )
                                                            .child(
                                                                svg::path()
                                                                    .attr("fill-rule", "evenodd")
                                                                    .attr("clip-rule", "evenodd")
                                                                    .attr(
                                                                        "d",
                                                                        "M4.293 4.293a1 1 0 \
                                                                         011.414 0L10 \
                                                                         8.586l4.293-4.293a1 1 0 \
                                                                         111.414 1.414L11.414 \
                                                                         10l4.293 4.293a1 1 0 \
                                                                         01-1.414 1.414L10 \
                                                                         11.414l-4.293 4.293a1 1 \
                                                                         0 01-1.414-1.414L8.586 \
                                                                         10 4.293 5.707a1 1 0 \
                                                                         010-1.414z",
                                                                    ),
                                                            ),
                                                        html::span()
                                                            .class("sr-only")
                                                            .child("Toggle sidebar"),
                                                    )),
                                                html::a()
                                                    .href("/frontend")
                                                    .class("flex items-center justify-between mr-4")
                                                    .child(
                                                        html::span()
                                                            .class(
                                                                "self-center text-2xl \
                                                                 font-semibold whitespace-nowrap \
                                                                 dark:text-white",
                                                            )
                                                            .child("Nghe"),
                                                    ),
                                            ),
                                        ),
                                        html::div().class("flex items-center lg:order-2").child((
                                            html::button()
                                                .r#type("button")
                                                .aria_controls("drawer-navigation")
                                                .attr("data-dropdown-toggle", "drawer-navigation")
                                                .class(
                                                    "p-2 mr-1 text-gray-500 rounded-lg md:hidden \
                                                     hover:text-gray-900 hover:bg-gray-100 \
                                                     dark:text-gray-400 dark:hover:text-white \
                                                     dark:hover:bg-gray-700 focus:ring-4 \
                                                     focus:ring-gray-300 dark:focus:ring-gray-600",
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
                                                                    "M8 4a4 4 0 100 8 4 4 0 \
                                                                     000-8zM2 8a6 6 0 1110.89 \
                                                                     3.476l4.817 4.817a1 1 0 \
                                                                     01-1.414 1.414l-4.816-4.\
                                                                     816A6 6 0 012 8z",
                                                                ),
                                                        ),
                                                    html::span()
                                                        .class("sr-only")
                                                        .child("Toggle search"),
                                                )),
                                            html::button()
                                                .id("user-menu-button")
                                                .r#type("button")
                                                .aria_expanded("false")
                                                .attr("data-dropdown-toggle", "dropdown")
                                                .class(
                                                    "flex mx-3 text-sm bg-gray-800 rounded-full \
                                                     md:mr-0 focus:ring-4 focus:ring-gray-300 \
                                                     dark:focus:ring-gray-600",
                                                )
                                                .child((
                                                    html::img()
                                                        .class("w-8 h-8 rounded-full")
                                                        .alt("user photo"),
                                                    html::span()
                                                        .class("sr-only")
                                                        .child("Open user menu"),
                                                )),
                                            html::div()
                                                .id("dropdown")
                                                .class(
                                                    "hidden z-50 my-4 w-56 text-base list-none \
                                                     bg-white rounded divide-y divide-gray-100 \
                                                     shadow dark:bg-gray-700 dark:divide-gray-600 \
                                                     rounded-xl",
                                                )
                                                .child((
                                                    html::div().class("py-3 px-4").child((
                                                        html::span()
                                                            .class(
                                                                "block text-sm font-semibold \
                                                                 text-gray-900 dark:text-white",
                                                            )
                                                            .child(user_info.username),
                                                        html::span()
                                                            .class(
                                                                "block text-sm text-gray-900 \
                                                                 truncate dark:text-white",
                                                            )
                                                            .child(user_info.email),
                                                    )),
                                                    html::ul()
                                                        .aria_labelledby("dropdown")
                                                        .class(
                                                            "py-1 text-gray-700 dark:text-gray-300",
                                                        )
                                                        .child(
                                                            html::li().child(
                                                                html::span()
                                                                    .class(
                                                                        "block py-2 px-4 text-sm \
                                                                         hover:bg-gray-100 \
                                                                         dark:hover:bg-gray-600 \
                                                                         dark:hover:text-white",
                                                                    )
                                                                    .child("Sign out"),
                                                            ),
                                                        ),
                                                )),
                                        )),
                                    )),
                            ),
                        html::aside()
                            .id("drawer-navigation")
                            .aria_label("Sidenav")
                            .class(
                                "fixed top-0 left-0 z-40 w-64 h-screen pt-14 transition-transform \
                                 -translate-x-full bg-white border-r border-gray-200 \
                                 md:translate-x-0 dark:bg-gray-800 dark:border-gray-700",
                            )
                            .child(html::div().class(
                                "overflow-y-auto py-5 px-3 h-full bg-white dark:bg-gray-800",
                            )),
                        html::main().class("p-4 md:ml-64 h-auto pt-20").child(child()),
                    ))
                })
            }))
            .build(),
    )
}
