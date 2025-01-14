use leptos::html;
use leptos::prelude::*;

pub fn Boundary(errors: ArcRwSignal<Errors>) -> impl IntoView {
    html::div().class(
        "flex items-center justify-center h-full w-full m-[-4] text-red-800 bg-red-50 \
         dark:bg-gray-800 dark:text-red-400",
    )
}
