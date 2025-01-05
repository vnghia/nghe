use leptos::prelude::*;
use leptos_router::components::Router;

use super::setup::Setup;

#[component]
pub fn Body() -> impl IntoView {
    view! {
        <Router>
            <Setup />
        </Router>
    }
}
