use std::path::{Path, PathBuf};

use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::*;
use crate::{DatabasePool, OSError};

pub async fn get_song_download_info(
    pool: &DatabasePool,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<(PathBuf, u64, u64)> {
    music_folders::table
        .inner_join(songs::table)
        .inner_join(user_music_folder_permissions::table)
        .filter(songs::id.eq(song_id))
        .filter(user_music_folder_permissions::user_id.eq(user_id))
        .filter(user_music_folder_permissions::allow)
        .select((music_folders::path, songs::relative_path, songs::file_hash, songs::file_size))
        .first::<(String, String, i64, i64)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song".into()).into())
        .map(|(music_folder_path, song_relative_path, song_file_hash, song_file_size)| {
            (
                Path::new(&music_folder_path).join(song_relative_path),
                song_file_hash as _,
                song_file_size as _,
            )
        })
}
