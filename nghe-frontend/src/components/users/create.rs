use leptos::html;
use leptos::prelude::*;
use nghe_api::user::create::Request;
use nghe_api::user::get::Response;

use crate::client::Client;
use crate::components::form;
use crate::{Error, flowbite};

pub const MODAL_ID: &str = "create-user-modal";

pub fn Modal(
    client: Client,
    users_resource: LocalResource<Result<Vec<Response>, Error>>,
) -> impl IntoView {
    let username = RwSignal::new(String::default());
    let email = RwSignal::new(String::default());
    let password = RwSignal::new(String::default());

    let (username_error, set_username_error) = signal(Option::default());
    let (email_error, set_email_error) = signal(Option::default());
    let (password_error, set_password_error) = signal(Option::default());

    let create_action = Action::<_, _, SyncStorage>::new_unsync(move |request: &Request| {
        let client = client.clone();
        let request = request.clone();
        async move {
            client.json(&request).await?;
            flowbite::modal::hide(MODAL_ID);
            users_resource.refetch();
            Ok(())
        }
    });

    html::div()
        .id(MODAL_ID)
        .tabindex("-1")
        .aria_hidden("true")
        .class(
            "hidden overflow-y-auto overflow-x-hidden fixed top-0 right-0 left-0 z-50 \
             justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full",
        )
        .child(html::div().class("relative p-4 w-full max-w-md max-h-full").child(form::Form(
            "Create new user",
            Some(MODAL_ID),
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
            "Submit",
            move |_| {
                let username = username();
                let username_error =
                    if username.is_empty() { Some("Username could not be empty") } else { None };
                set_username_error(username_error);

                let email = email();
                let email_error =
                    if email.is_empty() { Some("Email could not be empty") } else { None };
                set_email_error(email_error);

                let password = password();
                let password_error =
                    if password.is_empty() { Some("Password could not be empty") } else { None };
                set_password_error(password_error);

                if username_error.is_some() || email_error.is_some() || password_error.is_some() {
                    return;
                }
                create_action.dispatch(Request {
                    username,
                    password,
                    email,
                    role: nghe_api::user::Role { admin: false },
                    allow: false,
                });
            },
            create_action,
        )))
}
