use leptos::html;
use leptos::prelude::*;

use super::form;

pub fn Setup() -> impl IntoView {
    let username = RwSignal::new(String::default());
    let email = RwSignal::new(String::default());
    let password = RwSignal::new(String::default());

    let (username_error, set_username_error) = signal(Option::default());
    let (email_error, set_email_error) = signal(Option::default());
    let (password_error, set_password_error) = signal(Option::default());

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
                        let email_error =
                            if email.is_empty() { Some("Email could not be empty") } else { None };
                        set_email_error(email_error);

                        let password = password();
                        let password_error = if password.len() < 8 {
                            Some("Password must have at least 8 characters")
                        } else {
                            None
                        };
                        set_password_error(password_error);
                    },
                ),
            )),
    )
}
