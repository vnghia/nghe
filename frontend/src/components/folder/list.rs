use anyhow::Result;
use dioxus::prelude::*;
use nghe_types::music_folder::get_music_folder_ids::{
    GetMusicFolderIdsBody, GetMusicFolderIdsParams,
};
use nghe_types::music_folder::get_music_folder_stat::{
    GetMusicFolderStatBody, GetMusicFolderStatParams, MusicFolderStat,
};
use nghe_types::music_folder::remove_music_folder::{
    RemoveMusicFolderBody, RemoveMusicFolderParams,
};
use nghe_types::scan::get_scan_status::{GetScanStatusBody, GetScanStatusParams, ScanStatus};
use nghe_types::scan::start_scan::{ScanMode, StartScanBody, StartScanParams};
use readable::byte::*;
use readable::num::*;
use uuid::Uuid;

use super::super::Toast;
use crate::state::CommonState;
use crate::utils::modal::show_modal;
use crate::utils::time::DATETIME_FORMAT;
use crate::Route;

struct Folder {
    pub stat: MusicFolderStat,
    pub scan: Option<ScanStatus>,
}

async fn get_scan_status(common_state: &CommonState, id: Uuid) -> Result<Option<ScanStatus>> {
    common_state
        .send_with_common::<_, GetScanStatusBody>("/rest/getScanStatus", GetScanStatusParams { id })
        .await
        .map(|r| r.scan)
}

#[component]
pub fn Folders() -> Element {
    let nav = navigator();
    let common_state = CommonState::use_redirect();
    if !common_state()?.role.admin_role {
        nav.push(Route::Home {});
    }

    let mut folders: Signal<Vec<Folder>> = use_signal(Default::default);
    use_future(move || async move {
        if let Some(common_state) = common_state() {
            let ids = common_state
                .send_with_common::<_, GetMusicFolderIdsBody>(
                    "/rest/getMusicFolderIds",
                    GetMusicFolderIdsParams {},
                )
                .await
                .toast()
                .map_or_else(Default::default, |r| r.ids);

            for id in ids {
                let result: Result<()> = try {
                    folders.push(Folder {
                        stat: common_state
                            .send_with_common::<_, GetMusicFolderStatBody>(
                                "/rest/getMusicFolderStat",
                                GetMusicFolderStatParams { id },
                            )
                            .await?
                            .stat,
                        scan: get_scan_status(&common_state, id).await?,
                    })
                };
                if result.toast().is_none() {
                    break;
                }
            }
        }
    });

    let mut scan_idx: Signal<Option<usize>> = use_signal(Default::default);
    let mut scan_mode: Signal<Option<ScanMode>> = use_signal(Default::default);
    if let Some(idx) = scan_idx()
        && let Some(mode) = scan_mode()
        && let Some(common_state) = common_state()
        && idx < folders.len()
    {
        spawn(async move {
            scan_idx.set(None);
            scan_mode.set(None);
            let result: Result<()> = try {
                let id = folders.get(idx).expect("folder should not be none").stat.music_folder.id;
                folders.get_mut(idx).expect("folder stat should not be none").scan = common_state
                    .send_with_common::<_, StartScanBody>(
                        "/rest/startScan",
                        StartScanParams { id, mode },
                    )
                    .await?
                    .scan;
            };
            result.toast();
        });
    }

    let mut remove_idx: Signal<Option<usize>> = use_signal(Option::default);
    if let Some(idx) = remove_idx()
        && let Some(common_state) = common_state()
        && idx < folders.len()
    {
        spawn(async move {
            remove_idx.set(None);
            let folder = folders.remove(idx);
            common_state
                .send_with_common::<_, RemoveMusicFolderBody>(
                    "/rest/removeMusicFolder",
                    RemoveMusicFolderParams { id: folder.stat.music_folder.id },
                )
                .await
                .toast();
        });
    }

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
                                            d: "M12 10.5v6m3-3H9m4.06-7.19-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    tbody {
                        for (idx , folder) in folders.iter().enumerate() {
                            tr { key: "{folder.stat.music_folder.id}",
                                td { class: "text-base", "{folder.stat.music_folder.name}" }
                                td { class: "text-base", "{folder.stat.music_folder.path}" }
                                td { class: "text-base", "align": "right",
                                    "{Unsigned::from(folder.stat.artist_count)}"
                                }
                                td { class: "text-base", "align": "right",
                                    "{Unsigned::from(folder.stat.album_count)}"
                                }
                                td { class: "text-base", "align": "right",
                                    "{Unsigned::from(folder.stat.song_count)}"
                                }
                                td { class: "text-base", "align": "right",
                                    "{Unsigned::from(folder.stat.user_count)}"
                                }
                                td { class: "text-base", "align": "right",
                                    "{Byte::from(folder.stat.total_size)}"
                                }
                                td {
                                    class: "whitespace-nowrap",
                                    "align": "center",
                                    if folder.scan.is_some_and(|status| status.finished_at.is_none()) {
                                        button { class: "btn btn-ghost btn-xs",
                                            svg {
                                                class: "fill-none h-6 w-6 stroke-2 stroke-error",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "m9.75 9.75 4.5 4.5m0-4.5-4.5 4.5M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
                                                }
                                            }
                                        }
                                    } else {
                                        button {
                                            class: "btn btn-ghost btn-xs",
                                            onclick: move |_| {
                                                scan_idx.set(Some(idx));
                                                show_modal("scan-dialog").toast();
                                            },
                                            svg {
                                                class: "fill-none h-6 w-6 stroke-2 stroke-primary",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
                                                }
                                            }
                                        }
                                    }
                                    Link {
                                        to: Route::Folder {
                                            id: folder.stat.music_folder.id,
                                        },
                                        button { class: "btn btn-ghost btn-xs",
                                            svg {
                                                class: "fill-none h-6 w-6 stroke-2 stroke-primary",
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
                                    Link {
                                        to: Route::FolderPermission {
                                            id: folder.stat.music_folder.id,
                                        },
                                        button { class: "btn btn-ghost btn-xs",
                                            svg {
                                                class: "fill-none h-6 w-6 stroke-2 stroke-secondary",
                                                xmlns: "http://www.w3.org/2000/svg",
                                                view_box: "0 0 24 24",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
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
                                                d: "M15 13.5H9m4.06-7.19-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z"
                                            }
                                        }
                                    }
                                    br {}
                                    span { class: "badge badge-ghost",
                                        if let Some(status) = folder.scan {
                                            if let Some(finished_at) = status.finished_at {
                                                "Last scan at: {finished_at.format(DATETIME_FORMAT).unwrap()}"
                                            } else {
                                                "Scan started at: {status.started_at.format(DATETIME_FORMAT).unwrap()}"
                                            }
                                        } else {
                                            "No scan yet"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    dialog { id: "scan-dialog", class: "modal",
                        form {
                            class: "modal-box bg-base-100 flex gap-4 justify-center w-min",
                            method: "dialog",
                            div { class: "tooltip", "data-tip": "Full",
                                button {
                                    class: "btn btn-square btn-lg",
                                    onclick: move |_| scan_mode.set(Some(ScanMode::Full)),
                                    svg {
                                        class: "fill-none h-6 w-6 stroke-2 stroke-accent",
                                        xmlns: "http://www.w3.org/2000/svg",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            d: "m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
                                        }
                                    }
                                }
                            }
                            div { class: "tooltip", "data-tip": "Force",
                                button {
                                    class: "btn btn-square btn-lg",
                                    onclick: move |_| scan_mode.set(Some(ScanMode::Force)),
                                    svg {
                                        class: "fill-none h-6 w-6 stroke-2 stroke-error",
                                        xmlns: "http://www.w3.org/2000/svg",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            d: "m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607ZM10.5 7.5v6m3-3h-6"
                                        }
                                    }
                                }
                            }
                        }
                        form { class: "modal-backdrop", method: "dialog",
                            button {}
                        }
                    }
                }
            }
        }
    }
}
