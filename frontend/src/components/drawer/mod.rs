use dioxus::prelude::*;
use gloo::utils::{document, window};
use itertools::Itertools;
use strum::IntoEnumIterator;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use super::{DaisyTheme, Toast};
use crate::state::CommonState;
use crate::Route;

fn remove_focus() {
    if let Some(element) = document().active_element()
        && let Some(element) = element.dyn_ref::<HtmlElement>()
        && element.blur().is_err()
    {
        "can not blur active element".toast();
    }
}

#[component]
pub fn Drawer() -> Element {
    let mut common_state = CommonState::use_redirect();
    let mut global_theme = DaisyTheme::use_theme();

    let theme_classes = DaisyTheme::iter()
        .map(|t| (if t == global_theme() { "text-base btn-active" } else { "text-base" }, t))
        .collect_vec();

    let onclick_logout = move |_: Event<MouseData>| {
        common_state.set(None);
    };

    let common_state = common_state()?;

    rsx! {
        div { class: "drawer lg:drawer-open",
            input {
                r#type: "checkbox",
                class: "drawer-toggle",
                id: "main-drawer-toggle"
            }
            div { class: "max-h-screen drawer-content flex flex-col",
                div { class: "w-full navbar shadow bg-base-300 z-10",
                    div { class: "navbar-start",
                        div { class: "flex-none lg:hidden",
                            label {
                                aria_label: "open sidebar",
                                r#for: "main-drawer-toggle",
                                class: "btn btn-square btn-ghost",
                                svg {
                                    view_box: "fill-none 0 0 24 24",
                                    xmlns: "http://www.w3.org/2000/svg",
                                    class: "inline-block w-6 h-6 stroke-base-content stroke-2",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M4 6h16M4 12h16M4 18h16"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "navbar-center",
                        Link { class: "text-base-content btn btn-ghost text-xl", to: Route::Home {}, "nghe" }
                    }
                    div { class: "navbar-end",
                        div { class: "dropdown dropdown-end",
                            div {
                                class: "btn btn-ghost btn-circle flex justify-center items-center",
                                tabindex: "0",
                                role: "button",
                                svg {
                                    class: "h-8 w-8 stroke-base-content fill-none stroke-[1.5]",
                                    view_box: "0 0 24 24",
                                    "xmlns": "http://www.w3.org/2000/svg",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01"
                                    }
                                }
                            }
                            ul {
                                tabindex: "0",
                                class: "mt-3 z-[1] p-2 shadow menu menu-sm dropdown-content bg-base-300 rounded-box w-52",
                                div { class: "max-h-80 overflow-y-auto",
                                    for (class , theme) in theme_classes {
                                        li { key: "{theme.as_ref()}",
                                            button {
                                                class,
                                                onclick: move |_| {
                                                    global_theme.set(theme);
                                                    window().location().reload().expect("can not reload current page");
                                                },
                                                "{theme.as_ref()}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        button { class: "btn btn-ghost btn-circle",
                            svg {
                                class: "fill-none w-6 h-6 stroke-base-content stroke-2",
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                                }
                            }
                        }
                        div { class: "dropdown dropdown-end",
                            div {
                                class: "btn btn-ghost btn-circle flex justify-center items-center",
                                tabindex: "0",
                                role: "button",
                                svg {
                                    class: "fill-none w-8 h-8 stroke-base-content",
                                    xmlns: "http://www.w3.org/2000/svg",
                                    view_box: "0 0 24 24",
                                    stroke_width: "1.5",
                                    path {
                                        stroke_linejoin: "round",
                                        stroke_linecap: "round",
                                        d: "M17.982 18.725A7.488 7.488 0 0 0 12 15.75a7.488 7.488 0 0 0-5.982 2.975m11.963 0a9 9 0 1 0-11.963 0m11.963 0A8.966 8.966 0 0 1 12 21a8.966 8.966 0 0 1-5.982-2.275M15 9.75a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                                    }
                                }
                            }
                            ul {
                                tabindex: "0",
                                class: "mt-3 z-[1] p-2 shadow menu menu-sm dropdown-content bg-base-300 rounded-box w-52",
                                if common_state.role.admin_role {
                                    li {
                                        Link { class: "text-base", to: Route::Users {}, onclick: |_| { remove_focus() }, "Users" }
                                    }
                                    li {
                                        Link {
                                            class: "text-base",
                                            to: Route::Folders {},
                                            onclick: |_| { remove_focus() },
                                            "Folders"
                                        }
                                    }
                                }
                                li {
                                    button {
                                        class: "text-base",
                                        onclick: onclick_logout,
                                        "Logout"
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "min-h-0 w-full flex", Outlet::<Route> {} }
            }
            div { class: "drawer-side",
                label {
                    r#for: "main-drawer-toggle",
                    aria_label: "close sidebar",
                    class: "drawer-overlay"
                }
                ul { class: "menu p-4 w-80 min-h-full bg-base-200 text-base-content" }
            }
        }
    }
}
