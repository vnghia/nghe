use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use super::setup::Setup;

#[component]
pub fn Body() -> impl IntoView {
    Router(
        component_props_builder(&Router)
            .children(ToChildren::to_children(move || {
                Routes(
                    component_props_builder(&Routes)
                        .fallback(|| "Not found")
                        .children(ToChildren::to_children(move || {
                            Route(
                                component_props_builder(&Route)
                                    .path(path!("/setup"))
                                    .view(Setup)
                                    .build(),
                            )
                        }))
                        .build(),
                )
            }))
            .build(),
    )
}
