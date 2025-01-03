use leptos::prelude::*;
use leptos_router::components::Router;

use super::app::App;

#[component]
pub fn Body() -> impl IntoView {
    view! {
        <Router>
            <App />
        </Router>
    }
}
