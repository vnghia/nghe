use leptos::html;
use leptos::prelude::*;
use nghe_api::user::get::Request;

use super::navbar::Navbar;
use super::sidebar::Sidebar;
use crate::client::Client;
use crate::components::{Loading, init};

pub fn Shell<IV: IntoView + 'static>(
    child: impl Fn() -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let (client, _) = Client::use_client_redirect();
    let user = LocalResource::new(move || async move {
        let client = client().expect(Client::EXPECT_MSG);
        client.json(&Request).await.ok()
    });

    let node_ref = init::flowbite_suspense();
    Suspense(
        component_props_builder(&Suspense)
            .fallback(Loading)
            .children(ToChildren::to_children(move || {
                Suspend::new(async move {
                    let user = user.await;
                    user.map(|user| {
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
            .build(),
    )
}
