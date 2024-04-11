use dioxus::prelude::*;
use url::Url;

use super::ERROR_SIGNAL;

#[derive(Props, Clone, PartialEq)]
pub struct UserFormProps {
    title: &'static str,
    username: Signal<String>,
    password: Signal<String>,
    email: Option<Signal<String>>,
    server_url: Option<Signal<Url>>,
    submitable: Signal<bool>,
}

#[component]
pub fn UserForm(props: UserFormProps) -> Element {
    let UserFormProps { title, mut username, mut password, email, server_url, mut submitable } =
        props;
    let raw_url = server_url.as_ref().map(|url| use_signal(|| url().to_string()));

    let onclick = move |_: Event<MouseData>| {
        let username = username();
        if username.is_empty() {
            *ERROR_SIGNAL.write() = Some(anyhow::anyhow!("Username can not be empty"));
            return;
        }

        if let Some(email) = email {
            let email = email();
            if email.is_empty() {
                *ERROR_SIGNAL.write() = Some(anyhow::anyhow!("Email can not be empty"));
                return;
            }
        }

        let password = password();
        if password.is_empty() {
            *ERROR_SIGNAL.write() = Some(anyhow::anyhow!("Password can not be empty"));
            return;
        }

        if let Some(raw_url) = raw_url
            && let Some(mut server_url) = server_url
        {
            let raw_url = raw_url();
            if raw_url.is_empty() {
                *ERROR_SIGNAL.write() = Some(anyhow::anyhow!("Server url can not be empty"));
                return;
            }
            match Url::parse(&raw_url) {
                Ok(url) => server_url.set(url),
                Err(e) => {
                    *ERROR_SIGNAL.write() = Some(e.into());
                    return;
                }
            };
        }

        submitable.set(true);
    };

    rsx! {
        div { class: "bg-base-100 min-h-screen flex flex-col justify-center py-12 px-4 lg:px-8",
            div { class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 { class: "text-base-content mt-6 text-center text-3xl leading-9 font-extrabold",
                    "{title}"
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
                        if let Some(mut email) = email {
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
                        if let Some(mut raw_url) = raw_url {
                            div { class: "label",
                                span { class: "text-primary-content", "Server URL" }
                            }
                            input {
                                class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                                r#type: "text",
                                value: "{raw_url}",
                                oninput: move |e| raw_url.set(e.value())
                            }
                        }
                        button { class: "btn btn-active mt-8", onclick, "Submit" }
                    }
                }
            }
        }
    }
}
