use concat_string::concat_string;
use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use nghe_types::params::{to_password_token, CommonParams};
use nghe_types::system::ping::PingParams;
use rand::distributions::{Alphanumeric, DistString};
use url::Url;

use crate::state::{CommonState, COMMON_STATE_KEY};
use crate::Route;

#[component]
pub fn Login() -> Element {
    let nav = navigator();
    let mut common_state = use_synced_storage::<LocalStorage, Option<CommonState>>(
        COMMON_STATE_KEY.into(),
        Option::default,
    );

    let mut username = use_signal(String::default);
    let mut password = use_signal(String::default);
    let mut server_url = use_signal(|| Url::parse("http://localhost:3000").unwrap());

    let mut error_message = use_signal(String::default);

    let on_input_url = move |e: Event<FormData>| match Url::parse(&e.value()) {
        Ok(url) => {
            server_url.set(url);
            error_message.set(Default::default());
        }
        Err(e) => error_message.set(concat_string!("Can not parse server url: ", e.to_string())),
    };

    let on_submit_setup = move |_: Event<MouseData>| {
        let username = username();
        if username.is_empty() {
            error_message.set("Username can not be empty".into());
            return;
        }
        let password = password();
        if password.is_empty() {
            error_message.set("Password can not be empty".into());
            return;
        }

        spawn(async move {
            match try {
                let server_url = server_url();

                let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
                let token = to_password_token(password.as_bytes(), &salt);
                let common = CommonParams { username, salt, token };

                let client = reqwest::Client::new();
                client
                    .get(server_url.join("/rest/ping")?)
                    .query(&PingParams {}.with_common(&common))
                    .send()
                    .await?
                    .error_for_status()?;

                common_state.set(Some(CommonState { common, server_url }));
            } {
                Ok(()) => {}
                Err::<_, anyhow::Error>(e) => {
                    log::error!("{:?}", &e);
                    error_message.set(e.to_string());
                }
            }
        });
    };

    if common_state().is_some() {
        nav.push(Route::Home {});
    }

    rsx! {
        div { class: "bg-base-100 min-h-screen flex flex-col justify-center py-12 px-4 lg:px-8",
            div { class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 { class: "text-base-content mt-6 text-center text-3xl leading-9 font-extrabold",
                    "Login"
                }
            }
            div { class: "mt-8 sm:mx-auto sm:w-full sm:max-w-md",
                div { class: "bg-primary rounded-box py-8 px-6 shadow",
                    div { class: "form-control sm:mx-auto sm:w-full sm:max-w-md",
                        div { class: "label",
                            span { class: "text-primary-content", "Username" }
                        }
                        input {
                            class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                            r#type: "text",
                            value: "{username}",
                            autocomplete: "username",
                            oninput: move |e| username.set(e.value())
                        }
                        div { class: "label",
                            span { class: "text-primary-content", "Password" }
                        }
                        input {
                            class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                            r#type: "password",
                            value: "{password}",
                            autocomplete: "password",
                            oninput: move |e| password.set(e.value())
                        }
                        div { class: "label",
                            span { class: "text-primary-content", "Server URL" }
                        }
                        input {
                            class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                            r#type: "text",
                            value: "{server_url}",
                            oninput: on_input_url
                        }
                        button {
                            class: "btn btn-active mt-8",
                            onclick: on_submit_setup,
                            "Submit"
                        }
                    }
                }
            }
        }
        if !error_message().is_empty() {
            div { class: "toast",
                div { class: "alert alert-error", "{error_message}" }
            }
        }
    }
}
