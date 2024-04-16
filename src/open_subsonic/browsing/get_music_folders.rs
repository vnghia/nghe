use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};

use crate::models::*;
use crate::Database;

add_common_validate!(GetMusicFoldersParams);
add_axum_response!(GetMusicFoldersBody);

pub async fn get_music_folders_handler(
    State(database): State<Database>,
    req: GetMusicFoldersRequest,
) -> GetMusicFoldersJsonResponse {
    let music_folders = music_folders::table
        .inner_join(user_music_folder_permissions::table)
        .select(music_folders::MusicFolder::as_select())
        .filter(user_music_folder_permissions::user_id.eq(req.user_id))
        .load(&mut database.pool.get().await?)
        .await?
        .into_iter()
        .map(music_folders::MusicFolder::into)
        .collect();

    Ok(axum::Json(
        GetMusicFoldersBody { music_folders: MusicFolders { music_folder: music_folders } }.into(),
    ))
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
            GetMusicFoldersRequest::validated(
                GetMusicFoldersParams {},
                infra.user_id(0),
                Default::default(),
            ),
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
            GetMusicFoldersRequest::validated(
                GetMusicFoldersParams {},
                infra.user_id(0),
                Default::default(),
            ),
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
