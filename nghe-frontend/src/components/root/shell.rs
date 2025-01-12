use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use nghe_api::user::info::Request;

use super::navbar::Navbar;
use crate::client::Client;
use crate::components::{Loading, init};

pub fn Shell<IV: IntoView + 'static>(
    child: impl Fn() -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let (client, _) = Client::use_client_redirect();
    let user_info = LocalResource::new(move || async move {
        let client = client().expect(Client::EXPECT_MSG);
        client.json(&Request).await.unwrap()
    });
    Effect::new(move || {
        user_info.track();
        init::flowbite();
    });

    // TODO: rework after https://github.com/leptos-rs/leptos/issues/3481
    View::new(move || match user_info.get() {
        Some(user_info) => {
            let user_info = user_info.take();
            Either::Left(
                html::div().class("antialiased bg-gray-50 dark:bg-gray-900").child((
                    Navbar(user_info),
                    html::aside()
                        .id("drawer-navigation")
                        .aria_label("Sidenav")
                        .class(
                            "fixed top-0 left-0 z-40 w-64 h-screen pt-14 transition-transform \
                             -translate-x-full bg-white border-r border-gray-200 md:translate-x-0 \
                             dark:bg-gray-800 dark:border-gray-700",
                        )
                        .child(
                            html::div().class(
                                "overflow-y-auto py-5 px-3 h-full bg-white dark:bg-gray-800",
                            ),
                        ),
                    html::main().class("p-4 md:ml-64 h-auto pt-20").child(child()),
                )),
            )
        }
        None => Either::Right(Loading()),
    })
}
