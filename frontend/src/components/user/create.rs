use dioxus::prelude::*;
use nghe_types::user::create::{CreateUserParams, SubsonicCreateUserBody};
use nghe_types::user::Role;

use super::super::{Toast, UserForm};
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn CreateUser() -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let username = use_signal(String::default);
    let email = use_signal(String::default);
    let password = use_signal(String::default);
    let role = use_signal(|| Role {
        admin_role: false,
        stream_role: true,
        download_role: true,
        share_role: true,
    });
    let submitable = use_signal(bool::default);

    if submitable() {
        spawn(async move {
            let Role { admin_role, stream_role, download_role, share_role } = role();

            if common_state
                .unwrap()
                .send_with_common::<_, SubsonicCreateUserBody>(
                    "/rest/createUser",
                    CreateUserParams {
                        username: username(),
                        password: hex::encode(password()).into_bytes(),
                        email: email(),
                        admin_role,
                        stream_role,
                        download_role,
                        share_role,
                    },
                )
                .await
                .map_err(anyhow::Error::from)
                .toast()
                .is_some()
            {
                nav.go_back();
            };
        });
    }

    rsx! {
        UserForm {
            title: "Add a new user",
            username,
            password,
            email,
            role,
            submitable,
            grow_full_screen: false
        }
    }
}