use dioxus::prelude::*;
use nghe_types::user::setup::SetupParams;
use url::Url;

use super::ERROR_SIGNAL;
use crate::Route;

#[component]
pub fn Setup() -> Element {
    let nav = navigator();

    let mut username = use_signal(String::default);
    let mut email = use_signal(String::default);
    let mut password = use_signal(String::default);
    let mut server_url = use_signal(|| Url::parse("http://localhost:3000").unwrap());

    let on_input_url = move |e: Event<FormData>| match Url::parse(&e.value()) {
        Ok(url) => {
            server_url.set(url);
            *ERROR_SIGNAL.write() = None;
        }
        Err(e) => *ERROR_SIGNAL.write() = Some(e.into()),
    };

    let on_submit_setup = move |_: Event<MouseData>| {
        let username = username();
        if username.is_empty() {
            *ERROR_SIGNAL.write() = Some(anyhow::anyhow!("Username can not be empty"));
            return;
        }
        let email = email();
        if email.is_empty() {
            *ERROR_SIGNAL.write() = Some(anyhow::anyhow!("Email can not be empty"));
            return;
        }
        let password = password();
        if password.is_empty() {
            *ERROR_SIGNAL.write() = Some(anyhow::anyhow!("Password can not be empty"));
            return;
        }

        spawn(async move {
            match try {
                let server_url = server_url();
                let setup_params = SetupParams { username, email, password: password.into_bytes() };
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
    };

    rsx! {
        div { class: "bg-base-100 min-h-screen flex flex-col justify-center py-12 px-4 lg:px-8",
            div { class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 { class: "text-base-content mt-6 text-center text-3xl leading-9 font-extrabold",
                    "Setup an admin account"
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
                            span { class: "text-primary-content", "Email" }
                        }
                        input {
                            class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                            r#type: "email",
                            value: "{email}",
                            autocomplete: "email",
                            oninput: move |e| email.set(e.value())
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
    }
}
