use dioxus::prelude::*;

#[component]
pub fn Loading() -> Element {
    rsx! {
        div { class: "w-full h-full flex justify-center items-center",
            span { class: "w-10 h-10 loading loading-spinner loading-lg" }
        }
    }
}
