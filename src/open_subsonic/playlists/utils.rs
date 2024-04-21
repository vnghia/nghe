use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::id3::*;
use crate::models::*;
use crate::{DatabasePool, OSError};

pub async fn check_access_level(
    pool: &DatabasePool,
    playlist_id: Uuid,
    user_id: Uuid,
    minimum_level: playlists_users::AccessLevel,
) -> Result<()> {
    if playlists_users::table
        .filter(playlists_users::playlist_id.eq(playlist_id))
        .filter(playlists_users::user_id.eq(user_id))
        .select(playlists_users::access_level)
        .get_result::<playlists_users::AccessLevel>(&mut pool.get().await?)
        .await?
        < minimum_level
    {
        anyhow::bail!(OSError::Forbidden("access to playlist".into()))
    } else {
        Ok(())
    }
}

pub async fn add_playlist_user_unchecked(
    pool: &DatabasePool,
    playlist_id: Uuid,
    user_id: Uuid,
    access_level: playlists_users::AccessLevel,
) -> Result<()> {
    diesel::insert_into(playlists_users::table)
        .values(playlists_users::AddUser { playlist_id, user_id, access_level })
        .on_conflict((playlists_users::playlist_id, playlists_users::user_id))
        .do_update()
        .set(playlists_users::access_level.eq(access_level))
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

pub async fn get_playlist_id3_with_song_ids_unchecked(
    pool: &DatabasePool,
    playlist_id: Uuid,
    user_id: Uuid,
) -> Result<PlaylistId3WithSongIdsDb> {
    get_playlist_id3_with_song_ids_db(user_id)
        .filter(playlists::id.eq(playlist_id))
        .first(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Playlist".into()).into())
}

pub async fn add_songs(
    pool: &DatabasePool,
    playlist_id: Uuid,
    user_id: Uuid,
    song_ids: &[Uuid],
) -> Result<()> {
    check_access_level(pool, playlist_id, user_id, playlists_users::AccessLevel::Write).await?;

    // To ensure the insert order of these songs.
    for song_id in song_ids.iter().copied() {
        diesel::insert_into(playlists_songs::table)
            .values(playlists_songs::AddSong { playlist_id, song_id })
            .on_conflict_do_nothing()
            .execute(&mut pool.get().await?)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use nghe_types::playlists::create_playlist::CreatePlaylistParams;

    use super::super::create_playlist::create_playlist;
    use super::*;
    use crate::open_subsonic::playlists::add_playlist_user::add_playlist_user;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_check_access_level() {
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

        check_access_level(
            infra.pool(),
            playlist_id,
            infra.user_id(0),
            playlists_users::AccessLevel::Admin,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_check_access_level_not_found() {
        let infra =
            Infra::new().await.add_user(None).await.add_user(None).await.add_folder(true).await;
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

        assert!(
            check_access_level(
                infra.pool(),
                playlist_id,
                infra.user_id(1),
                playlists_users::AccessLevel::Read,
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_check_access_level_deny() {
        let infra =
            Infra::new().await.add_user(None).await.add_user(None).await.add_folder(true).await;
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

        add_playlist_user(
            infra.pool(),
            infra.user_id(0),
            playlist_id,
            infra.user_id(1),
            playlists_users::AccessLevel::Read,
        )
        .await
        .unwrap();

        assert!(matches!(
            check_access_level(
                infra.pool(),
                playlist_id,
                infra.user_id(1),
                playlists_users::AccessLevel::Write,
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::Forbidden(_)
        ));
    }
}
