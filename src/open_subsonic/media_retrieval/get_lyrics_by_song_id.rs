use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::open_subsonic::permission::with_permission;
use crate::{Database, DatabasePool};

add_common_validate!(GetLyricsBySongIdParams);
add_axum_response!(GetLyricsBySongIdBody);

async fn get_lyrics_by_song_id(
    pool: &DatabasePool,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<Vec<LyricId3Db>> {
    get_lyric_id3_db()
        .filter(with_permission(user_id))
        .filter(songs::id.eq(song_id))
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_lyrics_by_song_id_handler(
    State(database): State<Database>,
    req: GetLyricsBySongIdRequest,
) -> GetLyricsBySongIdJsonResponse {
    Ok(axum::Json(
        GetLyricsBySongIdBody {
            lyrics_list: LyricList {
                structured_lyrics: get_lyrics_by_song_id(
                    &database.pool,
                    req.user_id,
                    req.params.id,
                )
                .await?
                .into_iter()
                .map(LyricId3Db::into)
                .collect(),
            },
        }
        .into(),
    ))
}
