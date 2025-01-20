use leptos::prelude::*;
use leptos::{html, svg};
use nghe_api::user::get::Response;
use nghe_api::user::list::Request;

use crate::components::{Boundary, ClientRedirect, Loading, init};

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
            html::th().scope("col").class("px-6 py-3").child("Admin"),
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
            html::th()
                .scope("row")
                .class(
                    "flex items-center px-6 py-4 text-gray-900 whitespace-nowrap dark:text-white",
                )
                .child(html::div().child((
                    html::div().class("text-base font-semibold").child(user.username),
                    html::div().class("font-normal text-gray-500").child(user.email),
                ))),
            html::td().class("px-6 py-4").child(user.role.admin.then(|| {
                html::div().class("flex items-center justify-center").child(
                    html::span()
                        .class(
                            "inline-flex items-center justify-center w-6 h-6 me-2 text-sm \
                             font-semibold text-gray-800 bg-gray-100 rounded-full \
                             dark:bg-gray-700 dark:text-gray-300",
                        )
                        .child(
                            svg::svg()
                                .aria_hidden("true")
                                .attr("fill", "none")
                                .attr("viewBox", "0 0 16 12")
                                .attr("xmlns", "http://www.w3.org/2000/svg")
                                .class("w-2.5 h-2.5")
                                .child(
                                    svg::path()
                                        .attr("stroke", "currentColor")
                                        .attr("stroke-linecap", "round")
                                        .attr("stroke-linejoin", "round")
                                        .attr("stroke-width", "2")
                                        .attr("d", "M1 5.917 5.724 10.5 15 1.5"),
                                ),
                        ),
                )
            })),
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
                .class("w-full text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400")
                .child((Head(), Body(users))),
        )
}

pub fn Users() -> impl IntoView {
    ClientRedirect(move |client| {
        let (version, _) = signal(0_usize);
        let users = LocalResource::new(move || {
            version.track();
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
