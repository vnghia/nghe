use dioxus::prelude::*;
use nghe_types::music_folder::update_music_folder::{
    UpdateMusicFolderBody, UpdateMusicFolderParams,
};
use nghe_types::music_folder::FsType;
use uuid::Uuid;

use super::super::Toast;
use super::FolderForm;
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn Folder(id: Uuid) -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let name = use_signal(Default::default);
    let path = use_signal(Default::default);
    let fs_type = use_signal(|| FsType::Local);
    let mut submitable = use_signal(Default::default);

    if submitable()
        && let Some(common_state) = common_state()
    {
        spawn(async move {
            if common_state
                .send_with_common::<UpdateMusicFolderBody>(
                    "/rest/updateMusicFolder",
                    UpdateMusicFolderParams { id, name: name(), path: path(), fs_type: fs_type() },
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
            title: "Update music folder",
            name,
            path,
            fs_type,
            allow_empty: true,
            submitable
        }
    }
}
