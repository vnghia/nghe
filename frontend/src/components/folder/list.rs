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
                                th { class: "text-base", "Action" }
                            }
                        }
                        tbody {
                            for folder_stat in folder_stats {
                                tr { key: "{folder_stat.music_folder.id}",
                                    td { class: "text-base", "{folder_stat.music_folder.name}" }
                                    td { class: "text-base", "{folder_stat.music_folder.path}" }
                                    td { class: "text-base",
                                        "{Unsigned::from(folder_stat.artist_count)}"
                                    }
                                    td { class: "text-base",
                                        "{Unsigned::from(folder_stat.album_count)}"
                                    }
                                    td { class: "text-base",
                                        "{Unsigned::from(folder_stat.song_count)}"
                                    }
                                    td { class: "text-base",
                                        "{Unsigned::from(folder_stat.user_count)}"
                                    }
                                    td { class: "text-base", "{Byte::from(folder_stat.total_size)}" }
                                    td {
                                        Link {
                                            to: Route::Folder {
                                                id: folder_stat.music_folder.id,
                                            },
                                            button { class: "btn btn-ghost btn-circle",
                                                svg {
                                                    class: "fill-none h-6 w-6 stroke-[1.5] stroke-base-content",
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    view_box: "0 0 24 24",
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        d: "M20.71,7.04C21.1,6.65 21.1,6 20.71,5.63L18.37,3.29C18,2.9 17.35,2.9 16.96,3.29L15.12,5.12L18.87,8.87M3,17.25V21H6.75L17.81,9.93L14.06,6.18L3,17.25Z"
                                                    }
                                                }
                                            }
                                        }
                                    }
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
