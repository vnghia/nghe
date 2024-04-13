use dioxus::prelude::*;
use nghe_types::user::Role;
use url::Url;

use super::super::Toast;

#[derive(Props, Clone, PartialEq)]
pub struct UserFormProps {
    title: &'static str,
    username: Signal<String>,
    password: Signal<String>,
    email: Option<Signal<String>>,
    role: Option<Signal<Role>>,
    server_url: Option<Signal<Option<Url>>>,
    submitable: Signal<bool>,
    grow_full_screen: bool,
}

#[component]
pub fn UserForm(props: UserFormProps) -> Element {
    let UserFormProps {
        title,
        mut username,
        mut password,
        role,
        email,
        server_url,
        mut submitable,
        grow_full_screen,
    } = props;
    let raw_url = server_url
        .as_ref()
        .map(|s| use_signal(|| s().as_ref().map_or_else(Default::default, Url::to_string)));

    let onclick = move |_: Event<MouseData>| {
        let result: Result<(), anyhow::Error> = try {
            let username = username();
            if username.is_empty() {
                Err(anyhow::anyhow!("Username can not be empty"))?;
            }

            if let Some(email) = email {
                let email = email();
                if email.is_empty() {
                    Err(anyhow::anyhow!("Email can not be empty"))?;
                }
            }

            let password = password();
            if password.is_empty() {
                Err(anyhow::anyhow!("Password can not be empty"))?;
            }

            if let Some(raw_url) = raw_url
                && let Some(mut server_url) = server_url
            {
                server_url.set(
                    Some(raw_url())
                        .filter(|s| !s.is_empty())
                        .map(|s| Url::parse(&s))
                        .transpose()?,
                );
            }
            submitable.set(true);
        };
        result.toast();
    };

    let h_class = if grow_full_screen { "min-h-screen" } else { "h-full" };
    rsx! {
        div { class: "bg-base-100 {h_class} flex flex-col grow justify-center py-12 px-4 lg:px-8",
            div { class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 { class: "text-base-content mt-6 text-center text-3xl leading-9 font-extrabold",
                    "{title}"
                }
            }
            div { class: "mt-8 sm:mx-auto sm:w-full sm:max-w-md",
                div { class: "bg-primary rounded-box py-8 px-6 shadow",
                    div { class: "form-control sm:mx-auto sm:w-full sm:max-w-md",
                        div { class: "label",
                            span { class: "text-base text-primary-content", "Username" }
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
                                span { class: "text-base text-primary-content", "Email" }
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
                            span { class: "text-base text-primary-content", "Password" }
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
                                span { class: "text-base text-primary-content", "Server URL" }
                            }
                            input {
                                class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                                r#type: "text",
                                value: "{raw_url}",
                                oninput: move |e| raw_url.set(e.value())
                            }
                        }
                        if let Some(mut role) = role {
                            div { class: "flex flex-row justify-center items-center",
                                div { class: "grow flex flex-col justify-center items-center",
                                    div { class: "label",
                                        span { class: "text-base text-primary-content",
                                            "Admin"
                                        }
                                    }
                                    input {
                                        class: "toggle",
                                        r#type: "checkbox",
                                        checked: role().admin_role,
                                        oninput: move |e| {
                                            role.set(Role {
                                                admin_role: e.value().parse().unwrap(),
                                                ..role()
                                            })
                                        }
                                    }
                                }
                                div { class: "grow flex flex-col justify-center items-center",
                                    div { class: "label",
                                        span { class: "text-base text-primary-content",
                                            "Stream"
                                        }
                                    }
                                    input {
                                        class: "toggle",
                                        r#type: "checkbox",
                                        checked: role().stream_role,
                                        oninput: move |e| {
                                            role.set(Role {
                                                stream_role: e.value().parse().unwrap(),
                                                ..role()
                                            })
                                        }
                                    }
                                }
                                div { class: "grow flex flex-col justify-center items-center",
                                    div { class: "label",
                                        span { class: "text-base text-primary-content",
                                            "Download"
                                        }
                                    }
                                    input {
                                        class: "toggle",
                                        r#type: "checkbox",
                                        checked: role().download_role,
                                        oninput: move |e| {
                                            role.set(Role {
                                                download_role: e.value().parse().unwrap(),
                                                ..role()
                                            })
                                        }
                                    }
                                }
                                div { class: "grow flex flex-col justify-center items-center",
                                    div { class: "label",
                                        span { class: "text-base text-primary-content",
                                            "Share"
                                        }
                                    }
                                    input {
                                        class: "toggle",
                                        r#type: "checkbox",
                                        checked: role().share_role,
                                        oninput: move |e| {
                                            role.set(Role {
                                                share_role: e.value().parse().unwrap(),
                                                ..role()
                                            })
                                        }
                                    }
                                }
                            }
                        }
                        button {
                            class: "btn btn-active mt-8 text-base btn-accent text-accent-content",
                            onclick,
                            "Submit"
                        }
                    }
                }
            }
        }
    }
}
