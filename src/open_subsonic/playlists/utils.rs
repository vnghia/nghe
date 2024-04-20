use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::id3::*;
use crate::models::*;
use crate::open_subsonic::permission::with_permission;
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

pub async fn get_playlist_id3_with_song_ids(
    pool: &DatabasePool,
    user_id: Uuid,
    playlist_id: Uuid,
) -> Result<PlaylistId3WithSongIdsDb> {
    get_playlist_id3_with_song_ids_db()
        .filter(with_permission(user_id))
        .filter(playlists::id.eq(playlist_id))
        .first(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Playlist".into()).into())
}

pub async fn add_songs(
    pool: &DatabasePool,
    user_id: Uuid,
    playlist_id: Uuid,
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
