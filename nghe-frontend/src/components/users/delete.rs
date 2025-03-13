use leptos::html;
use leptos::prelude::*;
use nghe_api::user::create::Request;
use nghe_api::user::get::Response;
use wasm_bindgen::prelude::*;

use crate::Error;
use crate::client::Client;

pub const MODAL_ID: &str = "delete-user-modal";

#[wasm_bindgen(inline_js = "export function hideUserDeleteModal() { \
                            FlowbiteInstances.getInstance('Modal', 'delete-user-modal').hide(); }")]
extern "C" {
    fn hideUserDeleteModal();
}

pub fn Modal(
    client: Client,
    users_resource: LocalResource<Result<Vec<Response>, Error>>,
) -> impl IntoView {
    // let create_action = Action::<_, _, SyncStorage>::new_unsync(move |request: &Request| {
    //     let client = client.clone();
    //     let request = request.clone();
    //     async move {
    //         client.json(&request).await?;
    //         closeUserDeleteModal();
    //         users_resource.refetch();
    //         Ok(())
    //     }
    // });

    html::div()
        .id(MODAL_ID)
        .tabindex("-1")
        .aria_hidden("true")
        .class(
            "hidden overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 \
             justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full",
        )
        .child(html::div().class("relative p-4 w-full max-w-md max-h-full").child(html::div()))
}
