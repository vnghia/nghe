use leptos::html;
use leptos::prelude::*;

pub fn Sidebar() -> impl IntoView {
    html::aside()
        .id("drawer-navigation")
        .aria_label("Sidenav")
        .class(
            "fixed top-0 left-0 z-40 w-64 h-screen pt-14 transition-transform -translate-x-full \
             bg-white border-r border-gray-200 md:translate-x-0 dark:bg-gray-800 \
             dark:border-gray-700",
        )
        .child(
            html::div()
                .class("overflow-y-auto py-5 px-3 h-full bg-white dark:bg-gray-800")
                .child(html::ul().class("space-y-2")),
        )
}
