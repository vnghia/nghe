use dioxus::prelude::*;
use itertools::Itertools;
use nghe_types::user::get_users::{GetUsersParams, SubsonicGetUsersBody};

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

    let get_users_fut = use_resource(move || async move {
        common_state
            .unwrap()
            .send_with_common::<_, SubsonicGetUsersBody>("/rest/getUsers", GetUsersParams {})
            .await
    });

    match &*get_users_fut.read_unchecked() {
        Some(users) => {
            let users = &users
                .as_ref()
                .toast()?
                .root
                .body
                .users
                .iter()
                .sorted_by(|a, b| a.basic.username.cmp(&b.basic.username))
                .collect_vec();

            rsx! {
                div { class: "w-full h-full overflow-x-auto overflow-y-auto",
                    table { class: "table table-pin-rows",
                        thead {
                            tr { class: "shadow bg-base-200",
                                th { class: "text-base", "Username" }
                                th { class: "text-base", "Admin role" }
                                th { class: "text-base", "Stream role" }
                                th { class: "text-base", "Download role" }
                                th { class: "text-base", "Share role" }
                                th { class: "text-base", "Created at" }
                                th { class: "text-base", "Action" }
                            }
                        }
                        tbody {
                            for user in users {
                                tr { key: "{user.basic.username}",
                                    td { class: "text-base", "{user.basic.username}" }
                                    td {
                                        input {
                                            class: "rounded-badge checkbox",
                                            r#type: "checkbox",
                                            checked: user.basic.role.admin_role,
                                            disabled: true
                                        }
                                    }
                                    td {
                                        input {
                                            class: "rounded-badge checkbox",
                                            r#type: "checkbox",
                                            checked: user.basic.role.stream_role,
                                            disabled: true
                                        }
                                    }
                                    td {
                                        input {
                                            class: "rounded-badge checkbox",
                                            r#type: "checkbox",
                                            checked: user.basic.role.download_role,
                                            disabled: true
                                        }
                                    }
                                    td {
                                        input {
                                            class: "rounded-badge checkbox",
                                            r#type: "checkbox",
                                            checked: user.basic.role.share_role,
                                            disabled: true
                                        }
                                    }
                                    td { class: "text-base", "{user.created_at.date()}" }
                                    td {
                                        button { class: "btn btn-ghost btn-circle",
                                            svg {
                                                class: "fill-none h-6 w-6 stroke-2 stroke-error",
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
                                }
                            }
                        }
                        tfoot {
                            tr {
                                th { colspan: "7",
                                    Link { class: "w-full btn btn-circle", to: Route::CreateUser {},
                                        svg {
                                            class: "fill-none h-6 w-6 stroke-2 stroke-base-content",
                                            xmlns: "http://www.w3.org/2000/svg",
                                            view_box: "0 0 24 24",
                                            stroke: "currentColor",
                                            transform: "rotate(45)",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                d: "M6 18L18 6M6 6l12 12"
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
        None => rsx! {
            Loading {}
        },
    }
}
