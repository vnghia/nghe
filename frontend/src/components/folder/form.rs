use dioxus::prelude::*;

use super::super::Toast;
use crate::state::CommonState;
use crate::Route;

#[derive(Props, Clone, PartialEq)]
pub struct FolderFormProps {
    title: &'static str,
    name: Signal<Option<String>>,
    path: Signal<Option<String>>,
    allow: Option<Signal<bool>>,
    allow_empty: bool,
    submitable: Signal<bool>,
}

#[component]
pub fn FolderForm(props: FolderFormProps) -> Element {
    let FolderFormProps { title, mut name, mut path, allow, allow_empty, mut submitable } = props;

    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let mut raw_name = use_signal(String::default);
    let mut raw_path = use_signal(String::default);

    let onclick = move |_: Event<MouseData>| {
        let result: Result<(), anyhow::Error> = try {
            let raw_name = raw_name();
            name.set(if raw_name.is_empty() {
                if !allow_empty { Err(anyhow::anyhow!("Name can not be empty")) } else { Ok(None) }
            } else {
                Ok(Some(raw_name))
            }?);

            let raw_path = raw_path();
            path.set(if raw_path.is_empty() {
                if !allow_empty { Err(anyhow::anyhow!("Path can not be empty")) } else { Ok(None) }
            } else {
                Ok(Some(raw_path))
            }?);

            submitable.set(true);
        };
        result.toast();
    };

    let mt_class = if allow.is_some() { "mt-4" } else { "mt-8" };
    rsx! {
        div { class: "bg-base-100 h-full flex flex-col grow justify-center py-12 px-4 lg:px-8",
            div { class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 { class: "text-base-content mt-6 text-center text-3xl leading-9 font-extrabold",
                    "{title}"
                }
            }
            div { class: "mt-8 sm:mx-auto sm:w-full sm:max-w-md",
                div { class: "bg-base-300 rounded-box py-8 px-6 shadow",
                    div { class: "form-control sm:mx-auto sm:w-full sm:max-w-md",
                        div { class: "label",
                            span { class: "text-base text-base-content", "Name" }
                        }
                        input {
                            class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                            r#type: "text",
                            value: "{raw_name}",
                            autocomplete: "name",
                            oninput: move |e| raw_name.set(e.value())
                        }
                        div { class: "label",
                            span { class: "text-base text-base-content", "Path" }
                        }
                        input {
                            class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                            r#type: "text",
                            value: "{raw_path}",
                            autocomplete: "path",
                            oninput: move |e| raw_path.set(e.value())
                        }
                        if let Some(mut allow) = allow {
                            div { class: "flex flex-row justify-center items-center gap-4 mt-4",
                                input {
                                    class: "checkbox btn-xs",
                                    r#type: "checkbox",
                                    checked: allow(),
                                    oninput: move |e| allow.set(e.value().parse().unwrap())
                                }
                                div { class: "label",
                                    span { class: "text-base text-base-content", "Allow by default" }
                                }
                            }
                        }
                        button {
                            class: "btn {mt_class} btn-accent btn-outline",
                            onclick,
                            "Submit"
                        }
                    }
                }
            }
        }
    }
}
