use leptos::html;
use leptos::prelude::*;
use nghe_api::key::create::Request;

use crate::client::Client;
use crate::components::form;

pub fn Login() -> impl IntoView {
    let username = RwSignal::new(String::default());
    let password = RwSignal::new(String::default());

    let (username_error, set_username_error) = signal(Option::default());
    let (password_error, set_password_error) = signal(Option::default());

    let login_action =
        Action::<_, _, SyncStorage>::new_unsync(|(username, password): &(String, String)| {
            // let request = request.clone();
            async move {
                // Client::json(&request).await.map_err(|error| error.to_string())?;
                Ok::<_, String>(())
            }
        });

    html::section().class("bg-gray-50 dark:bg-gray-900").child(
        html::div()
            .class(
                "flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0",
            )
            .child((
                html::div()
                    .class(
                        "flex items-center mb-6 text-2xl font-semibold text-gray-900 \
                         dark:text-white",
                    )
                    .child("Nghe"),
                form::Form(
                    "Login",
                    move || {
                        (
                            form::input::Text(
                                "username",
                                "Username",
                                "username",
                                None,
                                None,
                                username,
                                username_error,
                            ),
                            form::input::Text(
                                "password",
                                "Password",
                                "password",
                                None,
                                None,
                                password,
                                password_error,
                            ),
                        )
                    },
                    "Login",
                    move |_| {
                        let username = username();
                        let username_error = if username.is_empty() {
                            Some("Username could not be empty")
                        } else {
                            None
                        };
                        set_username_error(username_error);

                        let password = password();
                        let password_error = if password.len() < 8 {
                            Some("Password must have at least 8 characters")
                        } else {
                            None
                        };
                        set_password_error(password_error);

                        if username_error.is_some() || password_error.is_some() {
                            return;
                        }
                        login_action.dispatch((username, password));
                    },
                    login_action,
                ),
            )),
    )
}
