use leptos::prelude::*;

use crate::components::ClientRedirect;

pub fn Home() -> impl IntoView {
    ClientRedirect(move |_| "Home")
}
