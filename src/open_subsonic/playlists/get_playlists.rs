use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use super::id3::*;
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetPlaylistsParams);
add_axum_response!(GetPlaylistsBody);

pub async fn get_playlists(pool: &DatabasePool, user_id: Uuid) -> Result<Vec<PlaylistId3Db>> {
    get_playlist_id3_db()
        .filter(playlists_users::user_id.eq(user_id))
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_playlists_handler(
    State(database): State<Database>,
    req: GetPlaylistsRequest,
) -> GetPlaylistsJsonResponse {
    Ok(axum::Json(
        GetPlaylistsBody {
            playlists: GetPlaylists {
                playlist: get_playlists(&database.pool, req.user_id)
                    .await?
                    .into_iter()
                    .map(PlaylistId3Db::into)
                    .collect(),
            },
        }
        .into(),
    ))
}
