use dioxus::prelude::*;
use nghe_types::params::{to_password_token, CommonParams};
use nghe_types::system::ping::PingParams;
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
    let server_url = use_signal(|| Url::parse("http://localhost").unwrap());
    let submitable = use_signal(bool::default);

    if submitable() {
        spawn(async move {
            let result: Result<_, anyhow::Error> = try {
                let server_url = server_url();

                let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                let token = to_password_token(password().as_bytes(), &salt);
                let common = CommonParams { username: username(), salt, token };

                let client = reqwest::Client::new();
                client
                    .get(server_url.join("/rest/ping")?)
                    .query(&PingParams {}.with_common(&common))
                    .send()
                    .await?
                    .error_for_status()?;

                common_state.set(Some(CommonState { common, server_url }));
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
