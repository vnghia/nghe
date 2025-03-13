mod create;
mod delete;

use leptos::prelude::*;
use leptos::{html, svg};
use nghe_api::user::get::Response;
use nghe_api::user::list::Request;

use crate::components::{Boundary, ClientRedirect, Loading, init};

fn Global() -> impl IntoView {
    html::div()
        .class(
            "flex flex-col items-end justify-between p-2.5 text-sm bg-white dark:bg-gray-800 \
             dark:border-gray-700 border-gray-200",
        )
        .child(
            html::div()
                .class(
                    "w-full md:w-auto flex flex-col md:flex-row space-y-2 md:space-y-0 \
                     items-stretch md:items-center justify-end flex-shrink-0",
                )
                .child(
                    html::button()
                        .r#type("button")
                        .attr("data-modal-target", create::MODAL_ID)
                        .attr("data-modal-toggle", create::MODAL_ID)
                        .class(
                            "flex items-center justify-center text-gray-900 whitespace-nowrap \
                             dark:text-white px-2 py-1.5 hover:bg-gray-100 dark:hover:bg-gray-600 \
                             focus:ring-3 focus:ring-gray-300 rounded-lg focus:outline-none \
                             dark:focus:ring-gray-600",
                        )
                        .child((
                            svg::svg()
                                .aria_hidden("true")
                                .class("h-3.5 w-3.5 mr-2")
                                .attr("fill", "currentColor")
                                .attr("viewBox", "0 0 20 20")
                                .attr("xmlns", "http://www.w3.org/2000/svg")
                                .child(
                                    svg::path()
                                        .attr("fill-rule", "evenodd")
                                        .attr("clip-rule", "evenodd")
                                        .attr(
                                            "d",
                                            "M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 \
                                             0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z",
                                        ),
                                ),
                            "New user",
                        )),
                ),
        )
}

fn Head() -> impl IntoView {
    html::thead()
        .class("text-xs text-gray-700 uppercase bg-gray-100 dark:bg-gray-700 dark:text-gray-400")
        .child(html::tr().child((
            html::th().scope("col").class("px-6 py-3").child("USERNAME"),
            html::th().scope("col").class("px-6 py-3").child("EMAIL"),
            html::th().scope("col").class("px-6 py-3").child("ADMIN"),
            html::th().scope("col").class("px-6 py-3").child("ACTION"),
        )))
}

fn Row(user: Response) -> impl IntoView {
    html::tr()
        .class(
            "bg-white border-b dark:bg-gray-800 dark:border-gray-700 border-gray-200 \
             hover:bg-gray-50 dark:hover:bg-gray-600",
        )
        .child((
            html::th()
                .scope("row")
                .class("px-6 py-3 font-medium text-gray-900 whitespace-nowrap dark:text-white")
                .child(user.username),
            html::td().class("px-6 py-3").child(user.email),
            html::td().class("px-6 py-3").role("admin").child(user.role.admin.then(|| {
                (
                    svg::svg()
                        .aria_hidden("true")
                        .attr("viewBox", "0 0 24 24")
                        .attr("fill", "none")
                        .attr("xmlns", "http://www.w3.org/2000/svg")
                        .class("w-5 h-5 text-green-600")
                        .child(
                            svg::path()
                                .attr("stroke", "currentColor")
                                .attr("stroke-linecap", "round")
                                .attr("stroke-width", "2")
                                .attr(
                                    "d",
                                    "M8.5 11.5 11 14l4-4m6 2a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z",
                                ),
                        ),
                    html::span().class("sr-only").child("Admin"),
                )
            })),
            html::td().class("flex items-center px-6 py-3").child((
                html::a()
                    .class("font-medium text-blue-600 dark:text-blue-500 hover:underline")
                    .child("Edit"),
                html::button()
                    .r#type("button")
                    .class("font-medium text-red-600 dark:text-red-500 hover:underline ms-3")
                    .child("Delete"),
            )),
        ))
}

fn Body(users: Vec<Response>) -> impl IntoView {
    html::tbody().child(For(component_props_builder(&For)
        .each(move || users.clone())
        .key(|user| user.id)
        .children(Row)
        .build()))
}

fn Table(node_ref: NodeRef<html::Div>, users: Vec<Response>) -> impl IntoView {
    html::div()
        .node_ref(node_ref)
        .class("m-4 relative overflow-x-auto shadow-md sm:rounded-lg")
        .child((
            Global(),
            html::table()
                .class("w-full text-sm text-left text-gray-500 dark:text-gray-400")
                .child((Head(), Body(users))),
        ))
}

pub fn Users() -> impl IntoView {
    ClientRedirect(move |client| {
        let local_client = client.clone();
        let users_resource = LocalResource::new(move || {
            let client = local_client.clone();
            async move { client.json(&Request).await.map(|response| response.users) }
        });

        let node_ref = init::flowbite_suspense();
        Suspense(
            component_props_builder(&Suspense)
                .fallback(Loading)
                .children(ToChildren::to_children(move || {
                    IntoRender::into_render(move || {
                        let client = client.clone();
                        Boundary(ToChildren::to_children(move || {
                            IntoRender::into_render(move || {
                                let client = client.clone();
                                Suspend::new(async move {
                                    users_resource.await.map(|users| {
                                        (
                                            Table(node_ref, users),
                                            create::Modal(client.clone(), users_resource),
                                            delete::Modal(client, users_resource),
                                        )
                                    })
                                })
                            })
                        }))
                    })
                }))
                .build(),
        )
    })
}
