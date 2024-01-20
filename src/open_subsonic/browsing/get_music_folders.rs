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
    use super::super::{refresh_permissions, test::setup_user_and_music_folders};
    use super::*;
    use crate::open_subsonic::common::request::CommonParams;
    use crate::utils::test::http::to_validated_form;
    use crate::utils::test::state::setup_state;

    use itertools::Itertools;

    #[tokio::test]
    async fn test_allow_all() {
        let (db, key, user_tokens, _temp_fs, music_folders, _) =
            setup_user_and_music_folders(2, 2, &[true, true, true, true]).await;

        refresh_permissions(
            db.get_pool(),
            None,
            Some(
                &music_folders
                    .iter()
                    .map(|music_folder| music_folder.id)
                    .collect_vec(),
            ),
        )
        .await
        .unwrap();

        let state = setup_state(db.get_pool(), key);

        for user_token in user_tokens {
            let form = to_validated_form(
                db.get_pool(),
                &key,
                GetMusicFoldersParams {
                    common: CommonParams {
                        username: user_token.0.username.clone(),
                        salt: user_token.1.clone(),
                        token: user_token.2,
                    },
                },
            )
            .await;

            let results = get_music_folders_handler(state.clone(), form)
                .await
                .unwrap()
                .0
                .root
                .music_folders
                .music_folder
                .into_iter()
                .sorted()
                .collect_vec();

            assert_eq!(
                results,
                music_folders.clone().into_iter().sorted().collect_vec()
            );
        }
    }

    #[tokio::test]
    async fn test_deny_some() {
        let (db, key, user_tokens, _temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(2, 2, &[true, false, true, true]).await;

        diesel::insert_into(user_music_folder_permissions::table)
            .values(&permissions[1])
            .execute(&mut db.get_pool().get().await.unwrap())
            .await
            .unwrap();

        refresh_permissions(
            db.get_pool(),
            None,
            Some(
                &music_folders
                    .iter()
                    .map(|music_folder| music_folder.id)
                    .collect_vec(),
            ),
        )
        .await
        .unwrap();

        let state = setup_state(db.get_pool(), key);

        {
            let form = to_validated_form(
                db.get_pool(),
                &key,
                GetMusicFoldersParams {
                    common: CommonParams {
                        username: user_tokens[0].0.username.clone(),
                        salt: user_tokens[0].1.clone(),
                        token: user_tokens[0].2,
                    },
                },
            )
            .await;

            let results = get_music_folders_handler(state.clone(), form)
                .await
                .unwrap()
                .0
                .root
                .music_folders
                .music_folder;

            assert_eq!(results, &music_folders[0..1]);
        }

        {
            let form = to_validated_form(
                db.get_pool(),
                &key,
                GetMusicFoldersParams {
                    common: CommonParams {
                        username: user_tokens[1].0.username.clone(),
                        salt: user_tokens[1].1.clone(),
                        token: user_tokens[1].2,
                    },
                },
            )
            .await;

            let results = get_music_folders_handler(state.clone(), form)
                .await
                .unwrap()
                .0
                .root
                .music_folders
                .music_folder
                .into_iter()
                .sorted()
                .collect_vec();

            assert_eq!(results, music_folders.into_iter().sorted().collect_vec());
        }
    }
}
