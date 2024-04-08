use concat_string::concat_string;
use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use nghe_types::open_subsonic::common::request::CommonParams;
use nghe_types::open_subsonic::system::ping::PingParams;
use nghe_types::utils::password::to_password_token;
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
                let common_params = CommonParams { username, salt, token };
                let ping_params = PingParams {}.with_common(common_params);

                let client = reqwest::Client::new();
                client
                    .get(server_url.join("/rest/ping")?)
                    .query(&ping_params)
                    .send()
                    .await?
                    .error_for_status()?;

                common_state.set(Some(CommonState { common: ping_params.common, server_url }));
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
      div { class: "min-h-screen flex flex-col justify-center py-12 px-4 lg:px-8",
        div { class: "sm:mx-auto sm:w-full sm:max-w-md",
          h2 { class: "mt-6 text-center text-3xl leading-9 font-extrabold text-gray-900",
            "Login"
          }
        }
        div { class: "mt-8 sm:mx-auto sm:w-full sm:max-w-md",
          div { class: "rounded-box bg-white py-8 px-6 shadow",
            div { class: "form-control sm:mx-auto sm:w-full sm:max-w-md",
              div { class: "label", span { "Username" } }
              input {
                class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                r#type: "text",
                value: "{username}",
                autocomplete: "username",
                oninput: move |e| username.set(e.value())
              }
              div { class: "label", span { "Password" } }
              input {
                class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                r#type: "password",
                value: "{password}",
                autocomplete: "password",
                oninput: move |e| password.set(e.value())
              }
              div { class: "label", span { "Server URL" } }
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
        div { class: "toast", div { class: "alert alert-error", "{error_message}" } }
      }
    }
}