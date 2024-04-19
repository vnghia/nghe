use anyhow::Result;
use axum::extract::State;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::id3::*;
use super::utils::get_playlist_id3_with_song_ids;
use crate::{Database, DatabasePool};

add_common_validate!(GetPlaylistParams);
add_axum_response!(GetPlaylistBody);

pub async fn get_playlist(
    pool: &DatabasePool,
    user_id: Uuid,
    playlist_id: Uuid,
) -> Result<PlaylistId3WithSongIdsDb> {
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
