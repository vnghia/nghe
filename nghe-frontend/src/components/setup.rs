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
                html::div()
                    .class(
                        "w-full bg-white rounded-lg shadow dark:border md:mt-0 sm:max-w-md xl:p-0 \
                         dark:bg-gray-800 dark:border-gray-700",
                    )
                    .child(
                        html::div().class("p-6 space-y-4 md:space-y-6 sm:p-8").child((
                            html::h1()
                                .class(
                                    "text-xl font-bold leading-tight tracking-tight text-gray-900 \
                                     md:text-2xl dark:text-white",
                                )
                                .child("Setup admin account"),
                            html::div().class("space-y-4 md:space-y-6").child((
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
                                html::button()
                                    .class(
                                        "w-full text-white bg-primary-600 hover:bg-primary-700 \
                                         focus:ring-4 focus:outline-none focus:ring-primary-300 \
                                         font-medium rounded-lg text-sm px-5 py-2.5 text-center \
                                         dark:bg-primary-600 dark:hover:bg-primary-700 \
                                         dark:focus:ring-primary-800",
                                    )
                                    .child("Setup"),
                            )),
                        )),
                    ),
            )),
    )
}
