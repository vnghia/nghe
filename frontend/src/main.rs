#![allow(non_snake_case)]

use dioxus::prelude::*;

const _TAILWIND_STYLE: &str = manganis::mg!(file("public/tailwind.css"));

fn main() {
    // Init debug
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    console_error_panic_hook::set_once();

    launch(App);
}

fn App() -> Element {
    rsx! {
        Router::<nghe_frontend::Route> {}
    }
}
