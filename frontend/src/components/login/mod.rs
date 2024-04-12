use dioxus::prelude::*;
use nghe_types::params::{to_password_token, CommonParams};
use nghe_types::user::login::{LoginParams, SubsonicLoginBody};
use rand::distributions::{Alphanumeric, DistString};
use url::Url;

use super::{Toast, UserForm};
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn Login() -> Element {
    let nav = navigator();
    let mut common_state = CommonState::use_no_redirect();

    let username = use_signal(String::default);
    let password = use_signal(String::default);
    let server_url = use_signal(Option::<Url>::default);
    let submitable = use_signal(bool::default);

    if submitable() {
        spawn(async move {
            let result: Result<_, anyhow::Error> = try {
                let server_url = server_url();

                let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                let token = to_password_token(password().as_bytes(), &salt);
                let common = CommonParams { username: username(), salt, token };
                let common_state_inner =
                    CommonState { common, server_url, role: Default::default() };
                let role = common_state_inner
                    .send_with_common::<_, SubsonicLoginBody>("/rest/login", LoginParams {})
                    .await?
                    .root
                    .body
                    .role;

                common_state.set(Some(CommonState { role, ..common_state_inner }));
            };
            result.toast();
        });
    }

    if common_state().is_some() {
        nav.push(Route::Home {});
    }

    rsx! {
        UserForm {
            title: "Login",
            username,
            password,
            server_url,
            submitable,
            grow_full_screen: true
        }
    }
}
