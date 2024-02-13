use crate::models::*;
use crate::{OSResult, ServerState};

use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::{Deserialize, Serialize};

#[add_validate]
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetMusicFoldersParams {}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolders {
    music_folder: Vec<music_folders::MusicFolder>,
}

#[wrap_subsonic_response]
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetMusicFoldersBody {
    music_folders: MusicFolders,
}

pub async fn get_music_folders_handler(
    State(state): State<ServerState>,
    req: GetMusicFoldersRequest,
) -> OSResult<GetMusicFoldersResponse> {
    let music_folders = music_folders::table
        .inner_join(user_music_folder_permissions::table)
        .select(music_folders::MusicFolder::as_select())
        .filter(user_music_folder_permissions::user_id.eq(req.user.id))
        .filter(user_music_folder_permissions::allow.eq(true))
        .load(&mut state.database.pool.get().await?)
        .await?;

    Ok(GetMusicFoldersBody {
        music_folders: MusicFolders {
            music_folder: music_folders,
        },
        ..Default::default()
    }
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::http::to_validated_form;
    use crate::utils::test::setup::setup_users_and_music_folders;
    use crate::utils::test::state::setup_state;

    use itertools::Itertools;

    #[tokio::test]
    async fn test_allow_all() {
        let (db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(2, 2, &[true, true, true, true]).await;

        let sorted_music_folders = music_folders.into_iter().sorted().collect_vec();

        for user in users {
            let form = to_validated_form(
                &db,
                GetMusicFoldersParams {
                    common: user.to_common_params(db.get_key()),
                },
            )
            .await;

            let state = setup_state(&db);
            let results = get_music_folders_handler(state, form)
                .await
                .unwrap()
                .0
                .root
                .music_folders
                .music_folder
                .into_iter()
                .sorted()
                .collect_vec();

            assert_eq!(&results, &sorted_music_folders);
        }
    }

    #[tokio::test]
    async fn test_deny_some() {
        let (db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(2, 2, &[true, false, true, true]).await;

        for (i, user) in users.into_iter().enumerate() {
            let form = to_validated_form(
                &db,
                GetMusicFoldersParams {
                    common: user.to_common_params(db.get_key()),
                },
            )
            .await;

            let state = setup_state(&db);
            let results = get_music_folders_handler(state, form)
                .await
                .unwrap()
                .0
                .root
                .music_folders
                .music_folder
                .into_iter()
                .sorted()
                .collect_vec();

            match i {
                0 => assert_eq!(results, &music_folders[0..1]),
                1 => assert_eq!(
                    results,
                    music_folders.clone().into_iter().sorted().collect_vec()
                ),
                _ => panic!(),
            };
        }
    }
}
