use leptos::html;
use leptos::prelude::*;
use nghe_api::user::delete::Request;
use nghe_api::user::get::Response;
use uuid::Uuid;

use crate::client::Client;
use crate::components::form;
use crate::{Error, flowbite};

pub const MODAL_ID: &str = "delete-user-modal";

pub fn Modal(
    client: Client,
    users_resource: LocalResource<Result<Vec<Response>, Error>>,
    get_delete_user_id: ReadSignal<Option<Uuid>>,
) -> impl IntoView {
    let delete_action = Action::<_, _>::new_unsync(move |request: &Request| {
        let client = client.clone();
        let request = *request;
        async move {
            client.json(&request).await?;
            flowbite::modal::hide(MODAL_ID);
            users_resource.refetch();
            Ok(())
        }
    });

    html::div()
        .id(MODAL_ID)
        .tabindex("-1")
        .aria_hidden("true")
        .class(
            "hidden overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 \
             justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full",
        )
        .child(html::div().class("relative p-4 w-full max-w-md max-h-full").child(form::Delete(
            "Are you sure you want to delete this user ?",
            MODAL_ID,
            move |_| {
                let user_id = get_delete_user_id();
                if let Some(user_id) = user_id {
                    delete_action.dispatch(Request { user_id });
                } else {
                    leptos::logging::warn!("User id is empty when opening detele user modal");
                }
            },
            delete_action,
        )))
}
