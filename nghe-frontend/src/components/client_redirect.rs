use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Redirect;

use crate::client::Client;

pub fn ClientRedirect<IV: IntoView + 'static>(
    child: impl Fn(Client) -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let client = Client::use_client();
    View::new(move || {
        if let Some(client) = client() {
            let child = child(client);
            Either::Left(child)
        } else {
            Redirect(component_props_builder(&Redirect).path("/login").build());
            Either::Right(())
        }
    })
}
