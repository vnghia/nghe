use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{
    add_axum_response, add_common_convert, add_common_validate, add_subsonic_response,
};
use serde::Serialize;

use crate::models::*;
use crate::Database;

#[add_common_convert]
pub struct GetMusicFoldersParams {}
add_common_validate!(GetMusicFoldersParams);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFolders {
    music_folder: Vec<music_folders::MusicFolder>,
}

#[add_subsonic_response]
#[derive(Debug)]
pub struct GetMusicFoldersBody {
    music_folders: MusicFolders,
}
add_axum_response!(GetMusicFoldersBody);

pub async fn get_music_folders_handler(
    State(database): State<Database>,
    req: GetMusicFoldersRequest,
) -> GetMusicFoldersJsonResponse {
    let music_folders = music_folders::table
        .inner_join(user_music_folder_permissions::table)
        .select(music_folders::MusicFolder::as_select())
        .filter(user_music_folder_permissions::user_id.eq(req.user_id))
        .filter(user_music_folder_permissions::allow)
        .load(&mut database.pool.get().await?)
        .await?;

    GetMusicFoldersBody { music_folders: MusicFolders { music_folder: music_folders } }.into()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_allow_all() {
        let infra = Infra::new().await.n_folder(2).await.add_user(None).await;

        let results = get_music_folders_handler(
            infra.state(),
            GetMusicFoldersParams {}.validated(infra.user_id(0)),
        )
        .await
        .unwrap()
        .0
        .root
        .body
        .music_folders
        .music_folder
        .into_iter()
        .map(|f| f.id)
        .sorted()
        .collect_vec();
        assert_eq!(results, infra.music_folder_ids(..));
    }

    #[tokio::test]
    async fn test_deny_some() {
        let infra = Infra::new().await.n_folder(2).await.add_user(None).await;
        infra.permissions(.., 1.., false).await;

        let results = get_music_folders_handler(
            infra.state(),
            GetMusicFoldersParams {}.validated(infra.user_id(0)),
        )
        .await
        .unwrap()
        .0
        .root
        .body
        .music_folders
        .music_folder
        .into_iter()
        .map(|f| f.id)
        .sorted()
        .collect_vec();
        assert_eq!(results, infra.music_folder_ids(..1));
    }
}
