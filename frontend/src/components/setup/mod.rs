use dioxus::prelude::*;
use nghe_types::user::setup::{SetupParams, SubsonicSetupBody};
use url::Url;

use super::{Toast, UserForm};
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn Setup() -> Element {
    let nav = navigator();

    let username = use_signal(String::default);
    let email = use_signal(String::default);
    let password = use_signal(String::default);
    let server_url = use_signal(Option::<Url>::default);
    let submitable = use_signal(bool::default);

    if submitable() {
        spawn(async move {
            let result: Result<_, anyhow::Error> = try {
                let server_url = server_url();

                CommonState::send::<_, SubsonicSetupBody>(
                    &server_url,
                    "rest/setup",
                    SetupParams {
                        username: username(),
                        email: email(),
                        password: password().into_bytes(),
                    },
                )
                .await?;
            };
            result.toast().and_then(|_| nav.push(Route::Login {}));
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
