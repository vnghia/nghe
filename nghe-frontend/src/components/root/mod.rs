mod navbar;
mod shell;
mod sidebar;

use leptos::prelude::*;

use crate::client::Client;
use crate::flowbite;

pub fn Root<IV: IntoView + 'static>(
    child: impl Fn() -> IV + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let location = leptos_router::hooks::use_location();
    Effect::new(move || {
        location.pathname.track();
        flowbite::init();
    });

    let client = Client::use_client();
    Show(
        component_props_builder(&Show)
            .when(move || client.with(Option::is_some))
            .children(ToChildren::to_children(move || shell::Shell(child)))
            .fallback(child)
            .build(),
    )
}
