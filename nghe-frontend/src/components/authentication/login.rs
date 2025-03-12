use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos_router::components::Redirect;
use nghe_api::key::create::Request;

use crate::client::Client;
use crate::components::form;

pub fn Login() -> impl IntoView {
    let (read_api_key, set_api_key) = Client::use_api_key();
    View::new(move || {
        if read_api_key.with(Option::is_some) {
            Redirect(component_props_builder(&Redirect).path("/").build());
            Either::Left(())
        } else {
            let username = RwSignal::new(String::default());
            let password = RwSignal::new(String::default());
            let client = RwSignal::new(nghe_api::constant::SERVER_NAME.into());

            let (username_error, set_username_error) = signal(Option::default());
            let (password_error, set_password_error) = signal(Option::default());
            let (client_error, set_client_error) = signal(Option::default());

            let login_action = Action::<_, _, SyncStorage>::new_unsync(move |request: &Request| {
                let request = request.clone();
                async move {
                    let api_key = Client::json_no_auth(&request).await?.api_key.api_key;
                    set_api_key(Some(api_key));
                    Ok(())
                }
            });

            Either::Right(
                html::section().class("bg-gray-50 dark:bg-gray-900 w-full").child(
                    html::div()
                        .class(
                            "flex flex-col items-center justify-center px-6 py-8 mx-auto \
                             md:h-screen lg:py-0",
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
                                        form::input::Text(
                                            "client",
                                            "Client",
                                            "text",
                                            None,
                                            None,
                                            client,
                                            client_error,
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
                                    let password_error = if password.is_empty() {
                                        Some("Password could not be empty")
                                    } else {
                                        None
                                    };
                                    set_password_error(password_error);

                                    let client = client();
                                    let client_error = if client.is_empty() {
                                        Some("Client could not be empty")
                                    } else {
                                        None
                                    };
                                    set_client_error(client_error);

                                    if username_error.is_some()
                                        || password_error.is_some()
                                        || client_error.is_some()
                                    {
                                        return;
                                    }
                                    login_action.dispatch(Request { username, password, client });
                                },
                                login_action,
                            ),
                        )),
                ),
            )
        }
    })
}
