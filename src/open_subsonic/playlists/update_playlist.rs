use anyhow::Result;
use axum::extract::State;
use diesel::{sql_types, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::utils::{add_songs, check_access_level};
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(UpdatePlaylistParams);
add_axum_response!(UpdatePlaylistBody);

async fn delete_by_indexes_unchecked(
    pool: &DatabasePool,
    playlist_id: Uuid,
    user_id: Uuid,
    song_indexes_to_remove: &[i64],
) -> Result<()> {
    diesel::sql_query(include_str!("delete_by_indexes.sql"))
        .bind::<sql_types::Uuid, _>(playlist_id)
        .bind::<sql_types::Uuid, _>(user_id)
        .bind::<sql_types::Array<sql_types::Int8>, _>(song_indexes_to_remove)
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

async fn update_playlist(
    pool: &DatabasePool,
    user_id: Uuid,
    UpdatePlaylistParams {
        playlist_id,
        name,
        comment,
        public,
        song_ids_to_add,
        song_indexes_to_remove,
    }: UpdatePlaylistParams,
) -> Result<()> {
    check_access_level(pool, playlist_id, user_id, playlists_users::AccessLevel::Write).await?;

    if name.is_some() || comment.is_some() || public.is_some() {
        diesel::update(playlists::table.filter(playlists::id.eq(playlist_id)))
            .set(playlists::UpdatePlaylist {
                name: name.map(|v| v.into()),
                comment: comment.map(|v| if v.is_empty() { None } else { Some(v.into()) }),
                public,
            })
            .execute(&mut pool.get().await?)
            .await?;
    }

    // removing before adding so we don't accidentally remove newly added songs.
    if let Some(song_indexes_to_remove) = song_indexes_to_remove {
        delete_by_indexes_unchecked(
            pool,
            playlist_id,
            user_id,
            &song_indexes_to_remove.iter().copied().map(|i| i as _).collect_vec(),
        )
        .await?
    }

    if let Some(song_ids_to_add) = song_ids_to_add {
        add_songs(pool, playlist_id, user_id, &song_ids_to_add).await?;
    }

    Ok(())
}

pub async fn update_playlist_handler(
    State(database): State<Database>,
    req: UpdatePlaylistRequest,
) -> UpdatePlaylistJsonResponse {
    update_playlist(&database.pool, req.user_id, req.params).await?;
    Ok(axum::Json(UpdatePlaylistBody {}.into()))
}

#[cfg(test)]
mod tests {
    use nghe_types::playlists::create_playlist::CreatePlaylistParams;
    use rand::prelude::SliceRandom;

    use super::super::utils::get_playlist_id3_with_song_ids_unchecked;
    use super::*;
    use crate::open_subsonic::playlists::create_playlist::create_playlist;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_delete_by_indexes_empty() {
        let infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: None,
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        delete_by_indexes_unchecked(infra.pool(), playlist_id, infra.user_id(0), &[1])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_by_indexes() {
        let n_song = 10_usize;

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: Some(song_fs_ids.clone()),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        delete_by_indexes_unchecked(infra.pool(), playlist_id, infra.user_id(0), &[1, 5])
            .await
            .unwrap();
        song_fs_ids.remove(4);
        song_fs_ids.remove(0);

        let song_ids =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .song_ids;
        assert_eq!(song_fs_ids, song_ids);
    }

    #[tokio::test]
    async fn test_delete_by_indexes_partial() {
        let n_song = 10_usize;

        let mut infra = Infra::new().await.add_user(None).await.n_folder(2).await;
        infra.add_n_song(0, n_song).add_n_song(1, n_song).scan(.., None).await;
        infra.remove_permission(None, 0).await;

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: Some(infra.song_ids(..).await),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        delete_by_indexes_unchecked(infra.pool(), playlist_id, infra.user_id(0), &[1, 5, 15])
            .await
            .unwrap();

        let mut song_fs_ids = infra.song_ids(1..).await;
        song_fs_ids.remove(4);
        song_fs_ids.remove(0);

        let song_ids =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .song_ids;
        assert_eq!(song_fs_ids, song_ids);
    }

    #[tokio::test]
    async fn test_update_playlist_basic() {
        let n_song = 10_usize;

        let mut infra = Infra::new().await.add_user(None).await.n_folder(2).await;
        infra.add_n_song(0, n_song).add_n_song(1, n_song).scan(.., None).await;
        infra.remove_permission(None, 0).await;

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: Some(infra.song_ids(..).await),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        let new_name = "new_playlist";
        let new_comment = "new_comment";

        update_playlist(
            infra.pool(),
            infra.user_id(0),
            UpdatePlaylistParams {
                playlist_id,
                name: Some(new_name.into()),
                comment: Some(new_comment.into()),
                public: Some(true),
                song_ids_to_add: None,
                song_indexes_to_remove: None,
            },
        )
        .await
        .unwrap();

        let playlist =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .playlist
                .basic;
        assert_eq!(playlist.name, new_name);
        assert_eq!(playlist.comment, Some(new_comment.into()));
        assert!(playlist.public);
    }

    #[tokio::test]
    async fn test_update_playlist_clear_comment() {
        let n_song = 10_usize;

        let mut infra = Infra::new().await.add_user(None).await.n_folder(2).await;
        infra.add_n_song(0, n_song).add_n_song(1, n_song).scan(.., None).await;
        infra.remove_permission(None, 0).await;

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: Some(infra.song_ids(..).await),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        let new_comment = "new_comment";

        update_playlist(
            infra.pool(),
            infra.user_id(0),
            UpdatePlaylistParams {
                playlist_id,
                name: None,
                comment: Some(new_comment.into()),
                public: None,
                song_ids_to_add: None,
                song_indexes_to_remove: None,
            },
        )
        .await
        .unwrap();

        let playlist =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .playlist
                .basic;
        assert_eq!(playlist.comment, Some(new_comment.into()));

        update_playlist(
            infra.pool(),
            infra.user_id(0),
            UpdatePlaylistParams {
                playlist_id,
                name: None,
                comment: Some(Default::default()),
                public: None,
                song_ids_to_add: None,
                song_indexes_to_remove: None,
            },
        )
        .await
        .unwrap();

        let playlist =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .playlist
                .basic;
        assert_eq!(playlist.comment, None);
    }

    #[tokio::test]
    async fn test_update_playlist_add_song() {
        let n_song = 10_usize;

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: Some(song_fs_ids[..5].to_vec()),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        update_playlist(
            infra.pool(),
            infra.user_id(0),
            UpdatePlaylistParams {
                playlist_id,
                name: None,
                comment: None,
                public: None,
                song_ids_to_add: Some(song_fs_ids[5..].to_vec()),
                song_indexes_to_remove: None,
            },
        )
        .await
        .unwrap();

        let song_ids =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .song_ids;
        assert_eq!(song_fs_ids, song_ids);
    }

    #[tokio::test]
    async fn test_update_playlist_remove_song() {
        let n_song = 10_usize;

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: Some(song_fs_ids.clone()),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        update_playlist(
            infra.pool(),
            infra.user_id(0),
            UpdatePlaylistParams {
                playlist_id,
                name: None,
                comment: None,
                public: None,
                song_ids_to_add: None,
                song_indexes_to_remove: Some(vec![1, 10]),
            },
        )
        .await
        .unwrap();

        song_fs_ids.remove(9);
        song_fs_ids.remove(0);

        let song_ids =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .song_ids;
        assert_eq!(song_fs_ids, song_ids);
    }

    #[tokio::test]
    async fn test_update_playlist_remove_song_add_song() {
        let n_song = 10_usize;

        let mut infra = Infra::new().await.add_user(None).await.add_folder(true).await;
        infra.add_n_song(0, n_song).scan(.., None).await;
        let mut song_fs_ids = infra.song_ids(..).await;
        song_fs_ids.shuffle(&mut rand::thread_rng());

        let playlist_id = create_playlist(
            infra.pool(),
            infra.user_id(0),
            &CreatePlaylistParams {
                name: Some("playlist".into()),
                playlist_id: None,
                song_ids: Some(song_fs_ids[..5].to_vec()),
            },
        )
        .await
        .unwrap()
        .playlist
        .basic
        .id;

        update_playlist(
            infra.pool(),
            infra.user_id(0),
            UpdatePlaylistParams {
                playlist_id,
                name: None,
                comment: None,
                public: None,
                song_ids_to_add: Some(song_fs_ids[5..].to_vec()),
                song_indexes_to_remove: Some(vec![1, 2, 7]),
            },
        )
        .await
        .unwrap();

        song_fs_ids.remove(1);
        song_fs_ids.remove(0);

        let song_ids =
            get_playlist_id3_with_song_ids_unchecked(infra.pool(), playlist_id, infra.user_id(0))
                .await
                .unwrap()
                .song_ids;
        assert_eq!(song_fs_ids, song_ids);
    }
}
