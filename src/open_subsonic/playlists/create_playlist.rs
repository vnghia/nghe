use anyhow::Result;
use axum::extract::State;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::id3::*;
use super::utils::{
    add_playlist_user_unchecked, add_songs, get_playlist_id3_with_song_ids_unchecked,
};
use crate::models::*;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(CreatePlaylistParams);
add_axum_response!(CreatePlaylistBody);

pub async fn create_playlist(
    pool: &DatabasePool,
    user_id: Uuid,
    CreatePlaylistParams { name, playlist_id, song_ids }: &CreatePlaylistParams,
) -> Result<PlaylistId3WithSongIdsDb> {
    let playlist_id = if let Some(name) = name.as_ref() {
        if song_ids.is_none() {
            let playlist = diesel::insert_into(playlists::table)
                .values(playlists::NewPlaylist { name: name.into() })
                .returning(BasicPlaylistId3Db::as_select())
                .get_result::<BasicPlaylistId3Db>(&mut pool.get().await?)
                .await?;
            add_playlist_user_unchecked(
                pool,
                playlist.id,
                user_id,
                playlists_users::AccessLevel::Admin,
            )
            .await?;
            return Ok(playlist.into());
        } else {
            let playlist_id = diesel::insert_into(playlists::table)
                .values(playlists::NewPlaylist { name: name.into() })
                .returning(playlists::id)
                .get_result::<Uuid>(&mut pool.get().await?)
                .await?;
            add_playlist_user_unchecked(
                pool,
                playlist_id,
                user_id,
                playlists_users::AccessLevel::Admin,
            )
            .await?;
            playlist_id
        }
    } else {
        playlist_id.ok_or_else(|| {
            OSError::InvalidParameter("either name or playlist id must be specified".into())
        })?
    };

    if let Some(song_ids) = song_ids {
        add_songs(pool, playlist_id, user_id, song_ids).await?;
    }
    get_playlist_id3_with_song_ids_unchecked(pool, playlist_id, user_id).await
}

pub async fn create_playlist_handler(
    State(database): State<Database>,
    req: CreatePlaylistRequest,
) -> CreatePlaylistJsonResponse {
    Ok(axum::Json(
        CreatePlaylistBody {
            playlist: create_playlist(&database.pool, req.user_id, &req.params)
                .await?
                .into(&database.pool)
                .await?,
        }
        .into(),
    ))
}

#[cfg(test)]
mod tests {
    use rand::seq::SliceRandom;

    use super::*;
    use crate::open_subsonic::playlists::utils::check_access_level;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_create_empty_playlist() {
        let playlist_name = "playlist";

        let infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        let PlaylistId3WithSongIdsDb { playlist, song_ids } = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: None,
            },
        )
        .await
        .unwrap();
        check_access_level(
            infra.pool(),
            playlist.basic.id,
            infra.user_id(0),
            playlists_users::AccessLevel::Admin,
        )
        .await
        .unwrap();

        assert_eq!(playlist.basic.name, playlist_name);
        assert!(!playlist.basic.public);
        assert_eq!(playlist.duration, 0_f32);
        assert_eq!(playlist.song_count, 0);

        assert!(song_ids.is_empty());
    }

    #[tokio::test]
    async fn test_create_playlist() {
        let n_song = 10_usize;
        let playlist_name = "playlist";

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).await.scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let PlaylistId3WithSongIdsDb { playlist, song_ids } = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: Some(song_fs_ids.clone()),
            },
        )
        .await
        .unwrap();
        check_access_level(
            infra.pool(),
            playlist.basic.id,
            infra.user_id(0),
            playlists_users::AccessLevel::Admin,
        )
        .await
        .unwrap();

        assert_eq!(playlist.basic.name, playlist_name);
        assert!(!playlist.basic.public);
        assert_eq!(playlist.song_count, song_ids.len() as i64);

        assert_eq!(song_fs_ids, song_ids);
    }

    #[tokio::test]
    async fn test_update_playlist() {
        let n_song = 10_usize;
        let playlist_name = "playlist";

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).await.scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: Some(song_fs_ids[..5].to_vec()),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        let PlaylistId3WithSongIdsDb { playlist, song_ids } = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: None,
                playlist_id: Some(playlist_id),
                song_ids: Some(song_fs_ids[5..].to_vec()),
            },
        )
        .await
        .unwrap();

        assert_eq!(playlist.song_count, song_ids.len() as i64);

        assert_eq!(song_fs_ids, song_ids);
    }

    #[tokio::test]
    async fn test_update_playlist_deny() {
        let n_song = 10_usize;
        let playlist_name = "playlist";

        let mut infra =
            Infra::new().await.add_user(None).await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).await.scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some(playlist_name.into()),
                playlist_id: None,
                song_ids: Some(song_fs_ids[..5].to_vec()),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        assert!(
            create_playlist(
                infra.pool(),
                infra.user_id(1),
                &CreatePlaylistParams {
                    name: None,
                    playlist_id: Some(playlist_id),
                    song_ids: Some(song_fs_ids[5..].to_vec()),
                },
            )
            .await
            .is_err()
        );
    }
}
