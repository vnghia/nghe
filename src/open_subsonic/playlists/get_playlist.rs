use anyhow::Result;
use axum::extract::State;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::id3::*;
use super::utils::{check_access_level, get_playlist_id3_with_song_ids};
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetPlaylistParams);
add_axum_response!(GetPlaylistBody);

pub async fn get_playlist(
    pool: &DatabasePool,
    user_id: Uuid,
    playlist_id: Uuid,
) -> Result<PlaylistId3WithSongIdsDb> {
    check_access_level(pool, playlist_id, user_id, playlists_users::AccessLevel::Read).await?;

    get_playlist_id3_with_song_ids(pool, user_id, playlist_id).await
}

pub async fn get_playlist_handler(
    State(database): State<Database>,
    req: GetPlaylistRequest,
) -> GetPlaylistJsonResponse {
    Ok(axum::Json(
        GetPlaylistBody {
            playlist: get_playlist(&database.pool, req.user_id, req.params.id)
                .await?
                .into(&database.pool)
                .await?,
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use nghe_types::playlists::create_playlist::CreatePlaylistParams;
    use rand::prelude::SliceRandom;

    use super::super::create_playlist::create_playlist;
    use super::*;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_get_playlist() {
        let n_song = 10_usize;
        let playlist_name = "playlist";

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: song_fs_ids.clone(),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        let PlaylistId3WithSongIdsDb { playlist, song_ids } =
            get_playlist(infra.pool(), infra.user_id(0), playlist_id).await.unwrap();

        assert_eq!(playlist.basic.name, playlist_name);
        assert!(!playlist.basic.public);
        assert_eq!(playlist.song_count, song_ids.len() as i64);

        assert_eq!(song_fs_ids, song_ids);
    }

    #[tokio::test]
    async fn test_get_playlist_partial() {
        let n_song = 10_usize;
        let playlist_name = "playlist";

        let mut infra = Infra::new().await.add_user(None).await.n_folder(2).await;
        infra.add_n_song(0, n_song).add_n_song(1, n_song).scan(.., None).await;
        infra.remove_permission(None, 1).await;

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: infra.song_ids(..).await,
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        let PlaylistId3WithSongIdsDb { playlist, song_ids } =
            get_playlist(infra.pool(), infra.user_id(0), playlist_id).await.unwrap();

        assert_eq!(playlist.basic.name, playlist_name);
        assert!(!playlist.basic.public);
        assert_eq!(playlist.song_count, song_ids.len() as i64);

        assert_eq!(infra.song_ids(..1).await, song_ids);
    }

    #[tokio::test]
    async fn test_get_playlist_deny() {
        let n_song = 10_usize;
        let playlist_name = "playlist";

        let mut infra =
            Infra::new().await.add_user(None).await.add_user(None).await.n_folder(1).await;
        infra.add_n_song(0, n_song).scan(.., None).await;

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(1),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: infra.song_ids(..).await,
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        assert!(get_playlist(infra.pool(), infra.user_id(0), playlist_id).await.is_err());
    }
}
