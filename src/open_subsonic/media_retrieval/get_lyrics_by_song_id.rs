use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{
    add_axum_response, add_common_convert, add_common_validate, add_subsonic_response,
};
use serde::Serialize;
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::common::id3::db::*;
use crate::open_subsonic::common::id3::query::*;
use crate::open_subsonic::common::id3::response::*;
use crate::open_subsonic::permission::with_permission;
use crate::{Database, DatabasePool};

#[add_common_convert]
#[derive(Debug)]
pub struct GetLyricsBySongIdParams {
    id: Uuid,
}
add_common_validate!(GetLyricsBySongIdParams);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricList {
    structured_lyrics: Vec<LyricId3>,
}

#[add_subsonic_response]
pub struct GetLyricsBySongIdBody {
    lyrics_list: LyricList,
}
add_axum_response!(GetLyricsBySongIdBody);

async fn get_lyrics_by_song_id(
    pool: &DatabasePool,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<Vec<LyricId3>> {
    get_lyric_id3_db()
        .filter(with_permission(user_id))
        .filter(songs::id.eq(song_id))
        .get_results(&mut pool.get().await.unwrap())
        .await?
        .into_iter()
        .map(LyricId3Db::into_res)
        .collect()
}

pub async fn get_lyrics_by_song_id_handler(
    State(database): State<Database>,
    req: GetLyricsBySongIdRequest,
) -> GetLyricsBySongIdJsonResponse {
    GetLyricsBySongIdBody {
        lyrics_list: LyricList {
            structured_lyrics: get_lyrics_by_song_id(&database.pool, req.user_id, req.params.id)
                .await?,
        },
    }
    .into()
}
