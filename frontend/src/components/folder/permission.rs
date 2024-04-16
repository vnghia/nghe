use std::collections::HashSet;

use dioxus::prelude::*;
use nghe_types::permission::add_permission::{AddPermissionParams, SubsonicAddPermissionBody};
use nghe_types::permission::get_allowed_users::{
    GetAllowedUsersParams, SubsonicGetAllowedUsersBody,
};
use nghe_types::permission::remove_permission::{
    RemovePermissionParams, SubsonicRemovePermissionBody,
};
use nghe_types::user::get_basic_user_ids::{GetBasicUserIdsParams, SubsonicGetBasicUserIdsBody};
use nghe_types::user::BasicUserId;
use uuid::Uuid;

use super::super::{Loading, Toast};
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn FolderPermission(id: Uuid) -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let mut users: Signal<Vec<(bool, BasicUserId)>> = use_signal(Default::default);
    use_future(move || async move {
        if let Some(common_state) = common_state() {
            let result: Result<_, anyhow::Error> = try {
                let allowed_ids = common_state
                    .send_with_common::<_, SubsonicGetAllowedUsersBody>(
                        "/rest/getAllowedUsers",
                        GetAllowedUsersParams { id },
                    )
                    .await?
                    .root
                    .body
                    .ids
                    .into_iter()
                    .collect::<HashSet<_>>();
                users.set(
                    common_state
                        .send_with_common::<_, SubsonicGetBasicUserIdsBody>(
                            "/rest/getBasicUserIds",
                            GetBasicUserIdsParams {},
                        )
                        .await?
                        .root
                        .body
                        .basic_user_ids
                        .into_iter()
                        .map(|u| (allowed_ids.contains(&u.id), u))
                        .collect(),
                )
            };
            result.toast();
        }
    });

    let mut toggle_idx: Signal<Option<(usize, bool)>> = use_signal(Option::default);
    if let Some((idx, allow)) = toggle_idx()
        && let Some(common_state) = common_state()
        && idx < users.len()
    {
        spawn(async move {
            toggle_idx.set(None);
            let user_id = users.get(idx).as_ref().unwrap().1.id;
            users.get_mut(idx).as_mut().unwrap().0 = allow;

            if allow {
                common_state
                    .send_with_common::<_, SubsonicAddPermissionBody>(
                        "/rest/addPermission",
                        AddPermissionParams { user_id: Some(user_id), music_folder_id: Some(id) },
                    )
                    .await
                    .toast();
            } else {
                common_state
                    .send_with_common::<_, SubsonicRemovePermissionBody>(
                        "/rest/addPermission",
                        RemovePermissionParams {
                            user_id: Some(user_id),
                            music_folder_id: Some(id),
                        },
                    )
                    .await
                    .toast();
            }
        });
    }

    if !users.is_empty() {
        rsx! {
            div { class: "w-full h-[calc(100%-4rem)] overflow-x-auto overflow-y-auto flex justify-center items-center my-8",
                div { class: "w-full sm:w-1/2 px-8",
                    table { class: "table table-pin-rows",
                        thead {
                            tr { class: "shadow bg-base-200",
                                th { class: "text-base", "Permission" }
                            }
                        }
                        tbody {
                            for (idx , user) in users.iter().enumerate() {
                                tr { key: "{user.1.id}",
                                    td { "align": "left",
                                        div { class: "flex flex-row gap-4",
                                            label { class: "swap",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: user.0,
                                                    oninput: move |e| { toggle_idx.set(Some((idx, e.value().parse().unwrap()))) }
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
                                            div { class: "label",
                                                span { class: "text-base text-base-content",
                                                    "{user.1.username}"
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
