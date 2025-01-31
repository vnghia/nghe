use leptos::prelude::*;
use leptos::{html, svg};
use nghe_api::user::get::Response;
use nghe_api::user::list::Request;

use crate::components::{Boundary, ClientRedirect, Loading, init};

fn ColGroup() -> impl IntoView {
    html::colgroup().child((html::col(), html::col(), html::col()))
}

fn Head() -> impl IntoView {
    html::thead()
        .class("text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400")
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
            html::th().scope("col").class("px-6 py-3").child("Name"),
            html::th().scope("col").class("px-6 py-3").child("Permissions"),
            html::th().scope("col").class("px-6 py-3").child("Actions"),
        )))
}

fn Row(user: Response) -> impl IntoView {
    html::tr()
        .class(
            "bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 \
             dark:hover:bg-gray-600",
        )
        .child((
            html::td().class("w-4 p-4").child((
                html::input().id("checkbox-all-search").r#type("checkbox").class(
                    "w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded \
                     focus:ring-blue-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 \
                     dark:focus:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 \
                     dark:border-gray-600",
                ),
                html::label().r#for("checkbox-all-search").class("sr-only").child("checkbox"),
            )),
            html::th().scope("row").class("flex items-center space-x-3 px-6 py-4").child((
                html::div().class("flex-1 min-w-0").child((
                    html::div()
                        .class("font-semibold text-gray-900 dark:text-white")
                        .child(user.username),
                    html::div()
                        .class("text-gray-500 truncate dark:text-gray-400")
                        .child(user.email),
                )),
                user.role.admin.then(|| {
                    html::span()
                        .class(
                            "inline-flex items-center bg-green-100 text-green-800 text-xs \
                             font-medium px-2.5 py-0.5 rounded-full dark:bg-green-900 \
                             dark:text-green-300",
                        )
                        .child("admin")
                }),
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
                .child((ColGroup(), Head(), Body(users))),
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
