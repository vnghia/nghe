use dioxus::prelude::*;
use nghe_types::music_folder::add_music_folder::{
    AddMusicFolderParams, SubsonicAddMusicFolderBody,
};

use super::super::Toast;
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn AddFolder() -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let mut name = use_signal(String::default);
    let mut path = use_signal(String::default);
    let mut allow = use_signal(|| true);
    let mut submitable = use_signal(bool::default);

    let onclick = move |_: Event<MouseData>| {
        let result: Result<(), anyhow::Error> = try {
            let name = name();
            if name.is_empty() {
                Err(anyhow::anyhow!("Name can not be empty"))?;
            }
            let path = path();
            if path.is_empty() {
                Err(anyhow::anyhow!("Path can not be empty"))?;
            }
            submitable.set(true);
        };
        result.toast();
    };

    if submitable()
        && let Some(common_state) = common_state()
    {
        spawn(async move {
            if common_state
                .send_with_common::<_, SubsonicAddMusicFolderBody>(
                    "/rest/addMusicFolder",
                    AddMusicFolderParams { name: name(), path: path(), permission: allow() },
                )
                .await
                .map_err(anyhow::Error::from)
                .toast()
                .is_some()
            {
                nav.push(Route::Folders {});
            } else {
                submitable.set(false);
            }
        });
    }

    rsx! {
        div { class: "bg-base-100 h-full flex flex-col grow justify-center py-12 px-4 lg:px-8",
            div { class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 { class: "text-base-content mt-6 text-center text-3xl leading-9 font-extrabold",
                    "Add music folder"
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
                            value: "{name}",
                            autocomplete: "name",
                            oninput: move |e| name.set(e.value())
                        }
                        div { class: "label",
                            span { class: "text-base text-base-content", "Path" }
                        }
                        input {
                            class: "input input-bordered sm:mx-auto sm:w-full sm:max-w-md",
                            r#type: "path",
                            value: "{path}",
                            autocomplete: "path",
                            oninput: move |e| path.set(e.value())
                        }
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
                        button {
                            class: "btn mt-4 btn-accent btn-outline",
                            onclick,
                            "Submit"
                        }
                    }
                }
            }
        }
    }
}
