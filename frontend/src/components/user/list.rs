use dioxus::prelude::*;
use itertools::Itertools;
use nghe_types::user::delete_user::{DeleteUserBody, DeleteUserParams};
use nghe_types::user::get_users::{GetUsersBody, GetUsersParams};
use nghe_types::user::User;

use super::super::{Loading, Toast};
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn Users() -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let mut users: Signal<Vec<User>> = use_signal(Default::default);
    use_future(move || async move {
        if let Some(common_state) = common_state() {
            users.set(
                common_state
                    .send_with_common::<_, GetUsersBody>("/rest/getUsers", GetUsersParams {})
                    .await
                    .toast()
                    .map_or_else(Default::default, |r| {
                        r.users
                            .into_iter()
                            .sorted_by(|a, b| a.basic.username.cmp(&b.basic.username))
                            .collect()
                    }),
            );
        }
    });

    let mut delete_idx: Signal<Option<usize>> = use_signal(Option::default);
    if let Some(idx) = delete_idx()
        && let Some(common_state) = common_state()
        && idx < users.len()
    {
        spawn(async move {
            delete_idx.set(None);
            let user = users.remove(idx);
            common_state
                .send_with_common::<_, DeleteUserBody>(
                    "/rest/deleteUser",
                    DeleteUserParams { id: user.id },
                )
                .await
                .toast();
        });
    }

    let common_state = common_state()?;

    if !users.is_empty() {
        rsx! {
            div { class: "w-full h-[calc(100%-4rem)] overflow-x-auto overflow-y-auto my-8",
                div { class: "min-w-full inline-block px-8",
                    table { class: "table table-pin-rows",
                        thead {
                            tr { class: "shadow bg-base-200",
                                th { class: "text-base", "align": "center", "Username" }

                                th { class: "text-base", "align": "center", "Admin" }
                                th { class: "text-base", "align": "center", "Stream" }
                                th { class: "text-base", "align": "center", "Download" }
                                th { class: "text-base", "align": "center", "Share" }
                                th { class: "text-base", "align": "center", "Created at" }
                                th { class: "text-base", "align": "center",
                                    Link { class: "btn btn-ghost btn-xs", to: Route::CreateUser {},
                                        svg {
                                            class: "fill-none h-6 w-6 stroke-2 stroke-accent",
                                            xmlns: "http://www.w3.org/2000/svg",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                d: "M18 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0ZM3 19.235v-.11a6.375 6.375 0 0 1 12.75 0v.109A12.318 12.318 0 0 1 9.374 21c-2.331 0-4.512-.645-6.374-1.766Z"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tbody {
                            for (idx , user) in users.iter().enumerate() {
                                tr { key: "{user.id}",
                                    td { class: "text-base", "{user.basic.username}" }
                                    td { "align": "center",
                                        label { class: "swap",
                                            input {
                                                r#type: "checkbox",
                                                checked: user.basic.role.admin_role,
                                                disabled: true
                                            }
                                            svg {
                                                class: "swap-on fill-none h-6 w-6 stroke-2 stroke-success",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "m4.5 12.75 6 6 9-13.5"
                                                }
                                            }
                                            svg {
                                                class: "swap-off fill-none h-6 w-6 stroke-2 stroke-error",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "M6 18L18 6M6 6l12 12"
                                                }
                                            }
                                        }
                                    }
                                    td { "align": "center",
                                        label { class: "swap",
                                            input {
                                                r#type: "checkbox",
                                                checked: user.basic.role.stream_role,
                                                disabled: true
                                            }
                                            svg {
                                                class: "swap-on fill-none h-6 w-6 stroke-2 stroke-success",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "m4.5 12.75 6 6 9-13.5"
                                                }
                                            }
                                            svg {
                                                class: "swap-off fill-none h-6 w-6 stroke-2 stroke-error",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "M6 18L18 6M6 6l12 12"
                                                }
                                            }
                                        }
                                    }
                                    td { "align": "center",
                                        label { class: "swap",
                                            input {
                                                r#type: "checkbox",
                                                checked: user.basic.role.download_role,
                                                disabled: true
                                            }
                                            svg {
                                                class: "swap-on fill-none h-6 w-6 stroke-2 stroke-success",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "m4.5 12.75 6 6 9-13.5"
                                                }
                                            }
                                            svg {
                                                class: "swap-off fill-none h-6 w-6 stroke-2 stroke-error",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "M6 18L18 6M6 6l12 12"
                                                }
                                            }
                                        }
                                    }
                                    td { "align": "center",
                                        label { class: "swap",
                                            input {
                                                r#type: "checkbox",
                                                checked: user.basic.role.share_role,
                                                disabled: true
                                            }
                                            svg {
                                                class: "swap-on fill-none h-6 w-6 stroke-2 stroke-success",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "m4.5 12.75 6 6 9-13.5"
                                                }
                                            }
                                            svg {
                                                class: "swap-off fill-none h-6 w-6 stroke-2 stroke-error",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "M6 18L18 6M6 6l12 12"
                                                }
                                            }
                                        }
                                    }
                                    td { class: "text-base", "align": "center",
                                        "{user.created_at.date()}"
                                    }
                                    td { "align": "center",
                                        if user.id != common_state.id {
                                            button {
                                                class: "btn btn-ghost btn-xs",
                                                onclick: move |_| { delete_idx.set(Some(idx)) },
                                                svg {
                                                    class: "fill-none h-6 w-6 stroke-2 stroke-error",
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    view_box: "0 0 24 24",
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        d: "M22 10.5h-6m-2.25-4.125a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0ZM4 19.235v-.11a6.375 6.375 0 0 1 12.75 0v.109A12.318 12.318 0 0 1 10.374 21c-2.331 0-4.512-.645-6.374-1.766Z"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            Loading {}
        }
    }
}
