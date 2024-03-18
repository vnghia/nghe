use crate::{models::*, DatabasePool, OSError};

use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub async fn get_song_download_info(
    pool: &DatabasePool,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<(Uuid, String)> {
    music_folders::table
        .inner_join(songs::table)
        .inner_join(user_music_folder_permissions::table)
        .filter(songs::id.eq(song_id))
        .filter(user_music_folder_permissions::user_id.eq(user_id))
        .filter(user_music_folder_permissions::allow)
        .select((songs::music_folder_id, songs::relative_path))
        .first::<(Uuid, String)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song".into()).into())
}

pub async fn get_song_stream_info(
    pool: &DatabasePool,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<(PathBuf, String)> {
    music_folders::table
        .inner_join(songs::table)
        .inner_join(user_music_folder_permissions::table)
        .filter(songs::id.eq(song_id))
        .filter(user_music_folder_permissions::user_id.eq(user_id))
        .filter(user_music_folder_permissions::allow)
        .select((music_folders::path, songs::relative_path, songs::format))
        .first::<(String, String, String)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song".into()).into())
        .map(|(music_folder_path, song_relative_path, song_format)| {
            (
                Path::new(&music_folder_path).join(song_relative_path),
                song_format,
            )
        })
}
