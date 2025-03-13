use leptos::prelude::*;
use leptos::{ev, html, svg};
use nghe_api::user::delete::Request;
use nghe_api::user::get::Response;

use crate::client::Client;
use crate::components::form;
use crate::{Error, flowbite};

pub const MODAL_ID: &str = "delete-user-modal";

pub fn Modal(
    client: Client,
    users_resource: LocalResource<Result<Vec<Response>, Error>>,
) -> impl IntoView {
    let delete_action = Action::<_, _, SyncStorage>::new_unsync(move |request: &Request| {
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
        .child(
            html::div().class("relative p-4 w-full max-w-md max-h-full").child(
                html::div().class("relative bg-white rounded-lg shadow-sm dark:bg-gray-700").child(
                    html::div().class("p-4 md:p-5 text-center").child((
                        svg::svg()
                            .aria_hidden("true")
                            .attr("fill", "none")
                            .attr("viewBox", "0 0 20 20")
                            .attr("xmlns", "http://www.w3.org/2000/svg")
                            .class("mx-auto mb-4 w-10 h-10 text-gray-500 dark:text-gray-400")
                            .child(
                                svg::path()
                                    .attr("stroke", "currentColor")
                                    .attr("stroke-linecap", "round")
                                    .attr("stroke-linejoin", "round")
                                    .attr("stroke-width", "2")
                                    .attr(
                                        "d",
                                        "M10 11V6m0 8h.01M19 10a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
                                    ),
                            ),
                        html::h3()
                            .class("mb-4 font-normal text-gray-900 dark:text-white")
                            .child("Are you sure you want to delete this user?"),
                        html::button()
                            .r#type("button")
                            .class(
                                "text-white bg-red-600 hover:bg-red-800 focus:ring-4 \
                                 focus:outline-none focus:ring-red-300 dark:focus:ring-red-800 \
                                 font-medium rounded-lg text-sm inline-flex items-center px-5 \
                                 py-2.5 text-center",
                            )
                            .child("Yes, I'm sure")
                            .on(ev::click, move |_| {
                                delete_action.dispatch(Request { user_id: uuid::Uuid::default() });
                            }),
                        html::button()
                            .r#type("button")
                            .attr("data-modal-hide", MODAL_ID)
                            .class(
                                "py-2.5 px-5 ms-3 text-sm font-medium text-gray-900 \
                                 focus:outline-none bg-white rounded-lg border border-gray-200 \
                                 hover:bg-gray-100 hover:text-blue-700 focus:z-10 focus:ring-4 \
                                 focus:ring-gray-100 dark:focus:ring-gray-700 dark:bg-gray-800 \
                                 dark:text-gray-400 dark:border-gray-600 dark:hover:text-white \
                                 dark:hover:bg-gray-700",
                            )
                            .child("No, cancel"),
                        form::error::Error(delete_action),
                    )),
                ),
            ),
        )
}
