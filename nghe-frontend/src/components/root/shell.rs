use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use nghe_api::user::info::Request;

use super::navbar::Navbar;
use super::sidebar::Sidebar;
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
            Either::Left(html::div().class("antialiased bg-gray-50 dark:bg-gray-900").child((
                Navbar(user_info),
                Sidebar(),
                html::main().class("p-4 md:ml-64 h-auto pt-20").child(child()),
            )))
        }
        None => Either::Right(Loading()),
    })
}
