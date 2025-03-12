use leptos::html;
use leptos::prelude::*;
use nghe_api::user::get::Request;

use super::navbar::Navbar;
use super::sidebar::Sidebar;
use crate::components::{Boundary, ClientRedirect, Loading, init};

pub fn Shell<IV: IntoView + 'static>(
    child: impl Fn() -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    ClientRedirect(move |client| {
        let user = LocalResource::new(move || {
            let client = client.clone();
            async move { client.json(&Request { id: None }).await }
        });
        let node_ref = init::flowbite_suspense();
        Suspense(
            component_props_builder(&Suspense)
                .fallback(Loading)
                .children(ToChildren::to_children(move || {
                    Boundary(ToChildren::to_children(move || {
                        Suspend::new(async move {
                            user.await.map(|user| {
                                let role = user.role;
                                html::div()
                                    .node_ref(node_ref)
                                    .class("antialiased bg-gray-50 dark:bg-gray-900 w-full")
                                    .child((
                                        Navbar(user),
                                        Sidebar(role),
                                        html::main().class("md:ml-64 pt-13 h-full").child(child()),
                                    ))
                            })
                        })
                    }))
                }))
                .build(),
        )
    })
}
