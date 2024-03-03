use crate::models::*;
use crate::Database;

use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;

#[add_validate]
pub struct GetMusicFoldersParams {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolders {
    music_folder: Vec<music_folders::MusicFolder>,
}

#[wrap_subsonic_response]
#[derive(Debug)]
pub struct GetMusicFoldersBody {
    music_folders: MusicFolders,
}

pub async fn get_music_folders_handler(
    State(database): State<Database>,
    req: GetMusicFoldersRequest,
) -> GetMusicFoldersJsonResponse {
    let music_folders = music_folders::table
        .inner_join(user_music_folder_permissions::table)
        .select(music_folders::MusicFolder::as_select())
        .filter(user_music_folder_permissions::user_id.eq(req.user.id))
        .filter(user_music_folder_permissions::allow.eq(true))
        .load(&mut database.pool.get().await?)
        .await?;

    GetMusicFoldersBody {
        music_folders: MusicFolders {
            music_folder: music_folders,
        },
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_subsonic::common::request::ValidatedForm;
    use crate::utils::test::setup::setup_users_and_music_folders;

    use itertools::Itertools;

    #[tokio::test]
    async fn test_allow_all() {
        let (temp_db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(2, 2, &[true, true, true, true]).await;

        let sorted_music_folders = music_folders.into_iter().sorted().collect_vec();

        for user in users {
            let form = ValidatedForm {
                params: GetMusicFoldersParams {},
                user,
            };

            let results = get_music_folders_handler(temp_db.state(), form)
                .await
                .unwrap()
                .0
                .root
                .body
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
        let (temp_db, users, _temp_fs, music_folders) =
            setup_users_and_music_folders(2, 2, &[true, false, true, true]).await;

        for (i, user) in users.into_iter().enumerate() {
            let form = ValidatedForm {
                params: GetMusicFoldersParams {},
                user,
            };

            let results = get_music_folders_handler(temp_db.state(), form)
                .await
                .unwrap()
                .0
                .root
                .body
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
