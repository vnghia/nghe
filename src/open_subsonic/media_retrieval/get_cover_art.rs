use std::path::Path;

use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use concat_string::concat_string;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl,
};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::add_validate;
use uuid::Uuid;

use super::super::common::id::{MediaType, MediaTypedId};
use crate::config::ArtConfig;
use crate::models::*;
use crate::open_subsonic::StreamResponse;
use crate::utils::fs::path::hash_size_to_path;
use crate::{Database, DatabasePool, OSError, ServerError};

#[add_validate]
#[derive(Debug)]
pub struct GetCoverArtParams {
    id: MediaTypedId,
}

pub async fn get_song_cover_art<P: AsRef<Path>>(
    pool: &DatabasePool,
    user_id: Uuid,
    cover_art_id: Uuid,
    song_art_dir: P,
) -> Result<StreamResponse> {
    let (file_format, file_hash, file_size) = music_folders::table
        .inner_join(songs::table)
        .inner_join(user_music_folder_permissions::table)
        .filter(user_music_folder_permissions::user_id.eq(user_id))
        .filter(user_music_folder_permissions::allow)
        .inner_join(
            song_cover_arts::table
                .on(song_cover_arts::id.eq(songs::cover_art_id.assume_not_null())),
        )
        .filter(song_cover_arts::id.eq(cover_art_id))
        .select((song_cover_arts::format, song_cover_arts::file_hash, song_cover_arts::file_size))
        .first::<(String, i64, i64)>(&mut pool.get().await?)
        .await
        .optional()?
        .ok_or_else(|| OSError::NotFound("Song cover art".into()))?;
    let art_path = hash_size_to_path(song_art_dir, file_hash as _, file_size as _)
        .join(concat_string!("cover.", file_format));
    StreamResponse::try_from_path(art_path).await
}

pub async fn get_cover_art_handler(
    State(database): State<Database>,
    Extension(art_config): Extension<ArtConfig>,
    req: GetCoverArtRequest,
) -> Result<StreamResponse, ServerError> {
    match req.params.id.t {
        Some(MediaType::Song) if let Some(song_path) = art_config.song_path => {
            get_song_cover_art(&database.pool, req.user_id, req.params.id.id, &song_path)
                .await
                .map_err(ServerError)
        }
        _ => Err(anyhow::anyhow!(OSError::NotFound("Cover art".into())).into()),
    }
}
