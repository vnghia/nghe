use crate::entity::{prelude::*, *};
use crate::{OSResult, ServerState};

use axum::extract::State;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use sea_orm::{EntityTrait, *};
use serde::{Deserialize, Serialize};

#[add_validate]
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetMusicFoldersParams {}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolders {
    music_folder: Vec<music_folder::Model>,
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
    let music_folders = MusicFolder::find()
        .join(
            JoinType::InnerJoin,
            music_folder::Relation::UserMusicFolder.def(),
        )
        .filter(
            Condition::all()
                .add(user_music_folder::Column::UserId.eq(req.user.id))
                .add(user_music_folder::Column::Allow.eq(true)),
        )
        .all(&state.conn)
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
    use super::super::{refresh_user_music_folders_all_users, test::setup_user_and_music_folders};
    use super::*;
    use crate::open_subsonic::common::request::CommonParams;
    use crate::utils::test::http::to_validated_form;
    use crate::utils::test::state::setup_state;

    fn sort_models(mut music_folders: Vec<music_folder::Model>) -> Vec<music_folder::Model> {
        music_folders.sort_by_key(|model| model.id);
        music_folders
    }

    #[tokio::test]
    async fn test_allow_all() {
        let (db, key, user_tokens, _temp_fs, music_folders, _) =
            setup_user_and_music_folders(2, 2, &[true, true, true, true]).await;
        refresh_user_music_folders_all_users(
            db.get_conn(),
            &music_folders
                .iter()
                .map(|music_folder| music_folder.id)
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();
        let music_folders = sort_models(music_folders);

        let state = setup_state(db.get_conn(), key);

        for user_token in user_tokens {
            let form = to_validated_form(
                db.get_conn(),
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
                .music_folder;
            let results = sort_models(results);

            assert_eq!(results, music_folders);
        }

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_deny_some() {
        let (db, key, user_tokens, _temp_fs, music_folders, permissions) =
            setup_user_and_music_folders(2, 2, &[true, false, true, true]).await;
        db.insert(permissions[1].clone().into_active_model()).await;
        refresh_user_music_folders_all_users(
            db.get_conn(),
            &music_folders
                .iter()
                .map(|music_folder| music_folder.id)
                .collect::<Vec<_>>(),
        )
        .await
        .unwrap();

        let state = setup_state(db.get_conn(), key);

        {
            let form = to_validated_form(
                db.get_conn(),
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
                db.get_conn(),
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
                .music_folder;

            assert_eq!(sort_models(results), sort_models(music_folders));
        }

        db.async_drop().await;
    }
}
