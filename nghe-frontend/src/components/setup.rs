use leptos::html;
use leptos::prelude::*;

use super::form;

pub fn Setup() -> impl IntoView {
    let username = RwSignal::new(String::default());
    let email = RwSignal::new(String::default());
    let password = RwSignal::new(String::default());

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
                                "username", "Username", "username", "username", "username",
                                username,
                            ),
                            form::input::Text(
                                "email",
                                "Email",
                                "email",
                                "email",
                                "email@example.com",
                                email,
                            ),
                            form::input::Text(
                                "password",
                                "Password",
                                "password",
                                "password",
                                "••••••••",
                                password,
                            ),
                        )
                    },
                    "Setup",
                ),
            )),
    )
}
