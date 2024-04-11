use dioxus::prelude::*;
use nghe_types::user::setup::SetupParams;
use url::Url;

use super::{UserForm, ERROR_SIGNAL};
use crate::Route;

#[component]
pub fn Setup() -> Element {
    let nav = navigator();

    let username = use_signal(String::default);
    let email = use_signal(String::default);
    let password = use_signal(String::default);
    let server_url = use_signal(|| Url::parse("http://localhost").unwrap());
    let submitable = use_signal(bool::default);

    if submitable() {
        spawn(async move {
            match try {
                let server_url = server_url();
                let setup_params = SetupParams {
                    username: username(),
                    email: email(),
                    password: password().into_bytes(),
                };
                let client = reqwest::Client::new();
                client
                    .get(server_url.join("/rest/setup")?)
                    .query(&setup_params)
                    .send()
                    .await?
                    .error_for_status()?;
            } {
                Ok(()) => {
                    nav.push(Route::Login {});
                }
                Err::<_, anyhow::Error>(e) => {
                    *ERROR_SIGNAL.write() = Some(e);
                }
            }
        });
    }

    rsx! {
        UserForm {
            title: "Setup an admin account",
            username,
            password,
            email,
            server_url,
            submitable
        }
    }
}
