use anyhow::Result;
use axum::extract::State;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::utils::check_access_level;
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(DeletePlaylistParams);
add_axum_response!(DeletePlaylistBody);

async fn delete_playlist(pool: &DatabasePool, user_id: Uuid, playlist_id: Uuid) -> Result<()> {
    check_access_level(pool, playlist_id, user_id, playlists_users::AccessLevel::Admin).await?;
    diesel::delete(playlists::table)
        .filter(playlists::id.eq(playlist_id))
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

pub async fn delete_playlist_handler(
    State(database): State<Database>,
    req: DeletePlaylistRequest,
) -> DeletePlaylistJsonResponse {
    delete_playlist(&database.pool, req.user_id, req.params.id).await?;
    Ok(axum::Json(DeletePlaylistBody {}.into()))
}

#[cfg(test)]
mod tests {
    use nghe_types::playlists::create_playlist::CreatePlaylistParams;

    use super::super::create_playlist::create_playlist;
    use super::*;
    use crate::open_subsonic::playlists::utils::get_playlist_id3_with_song_ids_unchecked;
    use crate::utils::test::Infra;
    use crate::OSError;

    #[tokio::test]
    async fn test_create_playlist() {
        let n_song = 10_usize;
        let playlist_name = "playlist";

        let mut infra = Infra::new().await.add_user(None).await.add_folder(0, true).await;
        infra.add_n_song(0, n_song).await.scan(.., None).await;

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: Some(infra.song_ids(..).await),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        delete_playlist(infra.pool(), infra.user_id(0), playlist_id).await.unwrap();
        assert!(matches!(
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::NotFound(_)
        ))
    }
}
