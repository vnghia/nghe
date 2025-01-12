use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use super::{Home, Root, authentication};

pub fn Body() -> impl IntoView {
    Router(
        component_props_builder(&Router)
            .base("/frontend")
            .children(ToChildren::to_children(move || {
                Root(move || {
                    Routes(
                        component_props_builder(&Routes)
                            .fallback(|| "Not found")
                            .children(ToChildren::to_children(move || {
                                (
                                    Route(
                                        component_props_builder(&Route)
                                            .path(path!(""))
                                            .view(Home)
                                            .build(),
                                    ),
                                    Route(
                                        component_props_builder(&Route)
                                            .path(path!("/setup"))
                                            .view(authentication::Setup)
                                            .build(),
                                    ),
                                    Route(
                                        component_props_builder(&Route)
                                            .path(path!("/login"))
                                            .view(authentication::Login)
                                            .build(),
                                    ),
                                )
                            }))
                            .build(),
                    )
                })
            }))
            .build(),
    )
}
