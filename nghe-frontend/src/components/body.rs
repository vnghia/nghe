use leptos::html;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use super::{Home, Loading, Root, Users, authentication};

pub fn Body() -> impl IntoView {
    html::div().class("flex h-dvh box-border").child(Router(
        component_props_builder(&Router)
            .base(nghe_api::common::FRONTEND_PREFIX)
            .children(ToChildren::to_children(move || {
                Root(move || {
                    Routes(
                        component_props_builder(&Routes)
                            .fallback(|| "Not found")
                            .transition(true)
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
                                            .path(path!("/loading"))
                                            .view(Loading)
                                            .build(),
                                    ),
                                    Route(
                                        component_props_builder(&Route)
                                            .path(path!("/users"))
                                            .view(Users)
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
    ))
}
