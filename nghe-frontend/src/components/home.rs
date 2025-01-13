use leptos::prelude::*;

use crate::client::Client;

pub fn Home() -> impl IntoView {
    let _ = Client::use_client_redirect();
    "Home"
}
