use leptos::html;
use leptos::prelude::*;
use nghe_api::user::info::Request;

use super::navbar::Navbar;
use super::sidebar::Sidebar;
use crate::client::Client;
use crate::components::error::Toast as _;
use crate::components::{Loading, init};

pub fn Shell<IV: IntoView + 'static>(
    child: impl Fn() -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let (client, _) = Client::use_client_redirect();
    let user_info = LocalResource::new(move || async move {
        let client = client().expect(Client::EXPECT_MSG);
        client.json(&Request).await.toast()
    });

    let node_ref = init::flowbite_suspense();
    Suspense(
        component_props_builder(&Suspense)
            .fallback(Loading)
            .children(ToChildren::to_children(move || {
                Suspend::new(async move {
                    let user_info = user_info.await;
                    user_info.map(|user_info| {
                        html::div()
                            .node_ref(node_ref)
                            .class("antialiased bg-gray-50 dark:bg-gray-900 w-full")
                            .child((
                                Navbar(user_info),
                                Sidebar(),
                                html::main().class("p-4 md:ml-64 pt-17 h-full").child(child()),
                            ))
                    })
                })
            }))
            .build(),
    )
}
