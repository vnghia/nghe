use concat_string::concat_string;
use leptos::prelude::*;
use leptos::{html, svg};
use nghe_api::user::get::Response;
use nghe_api::user::list::Request;

use crate::components::{Boundary, ClientRedirect, Loading, init};

fn Head() -> impl IntoView {
    html::thead()
        .class("text-xs text-gray-700 uppercase bg-gray-100 dark:bg-gray-700 dark:text-gray-400")
        .child(html::tr().child((
            html::th().scope("col").class("p-4").child(
                html::div().class("flex items-center").child((
                    html::input().id("checkbox-all-search").r#type("checkbox").class(
                        "w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded \
                         focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 \
                         dark:focus:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 \
                         dark:border-gray-600",
                    ),
                    html::label().r#for("checkbox-all-search").class("sr-only").child("checkbox"),
                )),
            ),
            html::th().scope("col").class("px-6 py-3").child("USERNAME"),
            html::th().scope("col").class("px-6 py-3").child("EMAIL"),
            html::th().scope("col").class("px-6 py-3").child("ADMIN"),
            html::th().scope("col").class("px-6 py-3").child("ACTION"),
        )))
}

fn Row(user: Response) -> impl IntoView {
    let checkbox_id = concat_string!("checkbox-user-", user.id.to_string());

    html::tr()
        .class(
            "bg-white border-b dark:bg-gray-800 dark:border-gray-700 border-gray-200 \
             hover:bg-gray-50 dark:hover:bg-gray-600",
        )
        .child((
            html::td().class("w-4 p-4").child(html::div().class("flex items-center").child((
                html::input().id(checkbox_id.clone()).r#type("checkbox").class(
                    "w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded-sm \
                     focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 \
                     dark:focus:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 \
                     dark:border-gray-600",
                ),
                html::label().r#for(checkbox_id.clone()).class("sr-only").child("checkbox"),
            ))),
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
            html::td().class("flex items-center px-6 py-4").child((
                html::a()
                    .class("font-medium text-blue-600 dark:text-blue-500 hover:underline")
                    .child("Edit"),
                html::a()
                    .class("font-medium text-red-600 dark:text-red-500 hover:underline ms-3")
                    .child("Remove"),
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

fn Table(users: Vec<Response>) -> impl IntoView {
    let node_ref = init::flowbite_suspense();
    html::div()
        .node_ref(node_ref)
        .class("m-4 relative overflow-x-auto shadow-md sm:rounded-lg")
        .child(
            html::table()
                .class("w-full text-sm text-left text-gray-500 dark:text-gray-400")
                .child((Head(), Body(users))),
        )
}

pub fn Users() -> impl IntoView {
    ClientRedirect(move |client| {
        let users = LocalResource::new(move || {
            let client = client.clone();
            async move { client.json(&Request).await.map(|response| response.users) }
        });

        Suspense(
            component_props_builder(&Suspense)
                .fallback(Loading)
                .children(ToChildren::to_children(move || {
                    Boundary(ToChildren::to_children(move || {
                        Suspend::new(async move { users.await.map(Table) })
                    }))
                }))
                .build(),
        )
    })
}
