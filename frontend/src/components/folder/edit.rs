use std::collections::HashSet;

use dioxus::prelude::*;
use nghe_types::permission::get_allowed_users::{
    GetAllowedUsersParams, SubsonicGetAllowedUsersBody,
};
use nghe_types::permission::set_permission::{SetPermissionParams, SubsonicSetPermissionBody};
use nghe_types::user::get_basic_user_ids::{GetBasicUserIdsParams, SubsonicGetBasicUserIdsBody};
use nghe_types::user::BasicUserId;
use uuid::Uuid;

use super::super::{Loading, Toast};
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn Folder(id: Uuid) -> Element {
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

            common_state
                .send_with_common::<_, SubsonicSetPermissionBody>(
                    "/rest/setPermission",
                    SetPermissionParams {
                        user_ids: vec![user_id],
                        music_folder_ids: vec![id],
                        allow,
                    },
                )
                .await
                .toast();
        });
    }

    if !users.is_empty() {
        rsx! {
            div { class: "w-full h-full overflow-x-auto overflow-y-auto",
                div { class: "min-w-full inline-block p-10",
                    table { class: "table table-pin-rows",
                        thead {
                            tr { class: "shadow bg-base-200",
                                th { class: "text-base", "Username" }
                                th { class: "text-base", "Allowed" }
                            }
                        }
                        tbody {
                            for (idx , user) in users.iter().enumerate() {
                                tr { key: "{user.1.id}",
                                    td { class: "text-base", "{user.1.username}" }
                                    td {
                                        input {
                                            class: "rounded-btn checkbox",
                                            oninput: move |e| { toggle_idx.set(Some((idx, e.value().parse().unwrap()))) },
                                            r#type: "checkbox",
                                            checked: user.0,
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
