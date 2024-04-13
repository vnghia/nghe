use dioxus::prelude::*;
use nghe_types::browsing::get_folder_stats::{GetFolderStatsParams, SubsonicGetFolderStatsBody};
use readable::byte::*;
use readable::num::*;

use super::super::{Loading, Toast};
use crate::state::CommonState;
use crate::Route;

#[component]
pub fn Folders() -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let get_folder_stats_fut = use_resource(move || async move {
        common_state
            .unwrap()
            .send_with_common::<_, SubsonicGetFolderStatsBody>(
                "/rest/getFolderStats",
                GetFolderStatsParams {},
            )
            .await
            .map(|r| r.root.body.folder_stats)
    });

    match &*get_folder_stats_fut.read_unchecked() {
        Some(r) => {
            let folder_stats = r.as_ref().toast()?;

            rsx! {
                div { class: "w-full h-full overflow-x-auto overflow-y-auto",
                    table { class: "table table-pin-rows",
                        thead {
                            tr { class: "shadow bg-base-200",
                                th { class: "text-base", "Name" }
                                th { class: "text-base", "Path" }
                                th { class: "text-base", "Artist count" }
                                th { class: "text-base", "Album count" }
                                th { class: "text-base", "Song count" }
                                th { class: "text-base", "User count" }
                                th { class: "text-base", "Total size" }
                            }
                        }
                        tbody {
                            for folder_stat in folder_stats {
                                tr { key: "{folder_stat.music_folder.id}",
                                    td { class: "text-base", "{folder_stat.music_folder.name}" }
                                    td { class: "text-base", "{folder_stat.music_folder.path}" }
                                    td { class: "text-base", "{Unsigned::from(folder_stat.artist_count)}" }
                                    td { class: "text-base", "{Unsigned::from(folder_stat.album_count)}" }
                                    td { class: "text-base", "{Unsigned::from(folder_stat.song_count)}" }
                                    td { class: "text-base", "{Unsigned::from(folder_stat.user_count)}" }
                                    td { class: "text-base", "{Byte::from(folder_stat.total_size)}" }
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {
            rsx! {
                Loading {}
            }
        }
    }
}
