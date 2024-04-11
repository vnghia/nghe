use dioxus::prelude::*;
use nghe_types::params::{to_password_token, CommonParams};
use nghe_types::system::ping::{PingParams, SubsonicPingBody};
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
                let common_state_inner = CommonState { common, server_url };
                common_state_inner
                    .send_with_common::<_, SubsonicPingBody>("/rest/ping", PingParams {})
                    .await?;

                common_state.set(Some(common_state_inner));
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
            email: None,
            server_url,
            submitable
        }
    }
}
