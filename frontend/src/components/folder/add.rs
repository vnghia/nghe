use dioxus::prelude::*;
use nghe_types::music_folder::add_music_folder::{AddMusicFolderBody, AddMusicFolderParams};

use super::super::Toast;
use super::FolderForm;
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn AddFolder() -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let name: Signal<Option<String>> = use_signal(Default::default);
    let path: Signal<Option<String>> = use_signal(Default::default);
    let allow = use_signal(|| true);
    let mut submitable = use_signal(Default::default);

    if submitable()
        && let Some(common_state) = common_state()
    {
        spawn(async move {
            if common_state
                .send_with_common::<_, AddMusicFolderBody>(
                    "/rest/addMusicFolder",
                    AddMusicFolderParams {
                        name: name().expect("name should not be none"),
                        path: path().expect("path should not be none"),
                        allow: allow(),
                    },
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
        FolderForm {
            title: "Add music folder",
            name,
            path,
            allow: Some(allow),
            allow_empty: false,
            submitable
        }
    }
}
