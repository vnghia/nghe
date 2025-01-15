use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos_router::components::Redirect;
use nghe_api::user::setup::Request;

use crate::client::Client;
use crate::components::form;

pub fn Setup() -> impl IntoView {
    let (read_api_key, _) = Client::use_api_key();
    View::new(move || {
        if read_api_key.with(Option::is_some) {
            Redirect(component_props_builder(&Redirect).path("/").build());
            Either::Left(())
        } else {
            let username = RwSignal::new(String::default());
            let email = RwSignal::new(String::default());
            let password = RwSignal::new(String::default());

            let (username_error, set_username_error) = signal(Option::default());
            let (email_error, set_email_error) = signal(Option::default());
            let (password_error, set_password_error) = signal(Option::default());

            let setup_action = Action::<_, _, SyncStorage>::new_unsync(|request: &Request| {
                let request = request.clone();
                async move {
                    Client::json_no_auth(&request).await?;
                    Ok(())
                }
            });

            Either::Right(View::new(move || {
                if setup_action.value().with(|result| result.as_ref().is_some_and(Result::is_ok)) {
                    Redirect(component_props_builder(&Redirect).path("/login").build());
                    Either::Left(())
                } else {
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
                                            "flex items-center mb-6 text-2xl font-semibold \
                                             text-gray-900 dark:text-white",
                                        )
                                        .child("Nghe"),
                                    form::Form(
                                        "Setup admin account",
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
                                                    "email",
                                                    "Email",
                                                    "email",
                                                    None,
                                                    "email@example.com",
                                                    email,
                                                    email_error,
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
                                        "Setup",
                                        move |_| {
                                            let username = username();
                                            let username_error = if username.is_empty() {
                                                Some("Username could not be empty")
                                            } else {
                                                None
                                            };
                                            set_username_error(username_error);

                                            let email = email();
                                            let email_error = if email.is_empty() {
                                                Some("Email could not be empty")
                                            } else {
                                                None
                                            };
                                            set_email_error(email_error);

                                            let password = password();
                                            let password_error = if password.len() < 8 {
                                                Some("Password must have at least 8 characters")
                                            } else {
                                                None
                                            };
                                            set_password_error(password_error);

                                            if username_error.is_some()
                                                || email_error.is_some()
                                                || password_error.is_some()
                                            {
                                                return;
                                            }
                                            setup_action.dispatch(Request {
                                                username,
                                                password,
                                                email,
                                            });
                                        },
                                        setup_action,
                                    ),
                                )),
                        ),
                    )
                }
            }))
        }
    })
}
