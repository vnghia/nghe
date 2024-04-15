use dioxus::prelude::*;
use itertools::Itertools;
use nghe_types::music_folder::get_music_folder_stats::{
    GetMusicFolderStatsParams, MusicFolderStat, SubsonicGetMusicFolderStatsBody,
};
use nghe_types::music_folder::remove_music_folder::{
    RemoveMusicFolderParams, SubsonicRemoveMusicFolderBody,
};
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

    let mut folder_stats: Signal<Vec<MusicFolderStat>> = use_signal(Default::default);
    use_future(move || async move {
        if let Some(common_state) = common_state() {
            folder_stats.set(
                common_state
                    .send_with_common::<_, SubsonicGetMusicFolderStatsBody>(
                        "/rest/getMusicFolderStats",
                        GetMusicFolderStatsParams {},
                    )
                    .await
                    .toast()
                    .map_or_else(Default::default, |r| {
                        r.root
                            .body
                            .folder_stats
                            .into_iter()
                            .sorted_by(|a, b| a.music_folder.name.cmp(&b.music_folder.name))
                            .collect()
                    }),
            );
        }
    });

    let mut remove_idx: Signal<Option<usize>> = use_signal(Option::default);
    if let Some(idx) = remove_idx()
        && let Some(common_state) = common_state()
        && idx < folder_stats.len()
    {
        spawn(async move {
            remove_idx.set(None);
            let folder_stat = folder_stats.remove(idx);
            common_state
                .send_with_common::<_, SubsonicRemoveMusicFolderBody>(
                    "/rest/removeMusicFolder",
                    RemoveMusicFolderParams { id: folder_stat.music_folder.id },
                )
                .await
                .toast();
        });
    }

    if !folder_stats.is_empty() {
        rsx! {
            div { class: "w-full h-[calc(100%-4rem)] overflow-x-auto overflow-y-auto my-8",
                div { class: "min-w-full inline-block px-8",
                    table { class: "table table-pin-rows",
                        thead {
                            tr { class: "shadow bg-base-200",
                                th { class: "text-base", "align": "center", "Name" }
                                th { class: "text-base", "align": "center", "Path" }
                                th { class: "text-base", "align": "center", "Artist" }
                                th { class: "text-base", "align": "center", "Album" }
                                th { class: "text-base", "align": "center", "Song" }
                                th { class: "text-base", "align": "center", "User" }
                                th { class: "text-base", "align": "center", "Size" }
                                th { class: "text-base", "align": "center",
                                    Link { class: "btn btn-ghost btn-xs", to: Route::AddFolder {},
                                        svg {
                                            class: "fill-none h-6 w-6 stroke-2 stroke-accent",
                                            xmlns: "http://www.w3.org/2000/svg",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                d: "M12 4.5v15m7.5-7.5h-15"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tbody {
                            for (idx , folder_stat) in folder_stats.iter().enumerate() {
                                tr { key: "{folder_stat.music_folder.id}",
                                class: "my-4",
                                    td { class: "text-base", "{folder_stat.music_folder.name}" }
                                    td { class: "text-base", "{folder_stat.music_folder.path}" }
                                    td { class: "text-base", "align": "right",
                                        "{Unsigned::from(folder_stat.artist_count)}"
                                    }
                                    td { class: "text-base", "align": "right",
                                        "{Unsigned::from(folder_stat.album_count)}"
                                    }
                                    td { class: "text-base", "align": "right",
                                        "{Unsigned::from(folder_stat.song_count)}"
                                    }
                                    td { class: "text-base", "align": "right",
                                        "{Unsigned::from(folder_stat.user_count)}"
                                    }
                                    td { class: "text-base", "align": "right",
                                        "{Byte::from(folder_stat.total_size)}"
                                    }
                                    td { "align": "center",
                                        Link {
                                            to: Route::Folder {
                                                id: folder_stat.music_folder.id,
                                            },
                                            button { class: "btn btn-ghost btn-xs",
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
                                        button {
                                            class: "btn btn-ghost btn-xs",
                                            onclick: move |_| { remove_idx.set(Some(idx)) },
                                            svg {
                                                class: "fill-none h-6 w-6 stroke-2 stroke-error",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "M6 18L18 6M6 6l12 12"
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
    } else {
        rsx! {
            Loading {}
        }
    }
}
