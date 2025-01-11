mod shell;

use leptos::prelude::*;
use nghe_api::user::info::Request;
use wasm_bindgen::prelude::*;

use crate::client::Client;

#[wasm_bindgen(inline_js = "export function initializeFlowbite() { initFlowbite(); }")]
extern "C" {
    fn initializeFlowbite();
}

pub fn Root<IV: IntoView + 'static>(
    child: impl Fn() -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let location = leptos_router::hooks::use_location();
    Effect::new(move |_| {
        location.pathname.track();
        initializeFlowbite();
    });

    let (client, _effect) = Client::use_client();
    let user_info = LocalResource::new(move || async move {
        if let Some(client) = client() {
            Some(client.json(&Request).await.map_err(|error| error.to_string()))
        } else {
            None
        }
    });

    Show(
        component_props_builder(&Show)
            .when(move || {
                location
                    .pathname
                    .with(|pathname| pathname != "/frontend/setup" && pathname != "/frontend/login")
            })
            .children(ToChildren::to_children(move || {
                Suspend::new(async move {
                    let user_info = user_info.await.unwrap().unwrap();
                    shell::Shell(user_info, child)
                })
            }))
            .fallback(child)
            .build(),
    )
}
