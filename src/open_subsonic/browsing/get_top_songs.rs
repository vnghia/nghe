use anyhow::Result;
use axum::extract::State;
use diesel::dsl::sum;
use diesel::{ExpressionMethods, JoinOnDsl, PgSortExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{stream, StreamExt, TryStreamExt};
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::id3::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetTopSongsParams);
add_axum_response!(GetTopSongsBody);

async fn get_top_songs(
    pool: &DatabasePool,
    user_id: Uuid,
    artist: String,
    count: Option<u32>,
) -> Result<Vec<SongId3Db>> {
    get_song_id3_db()
        .inner_join(artists::table.on(artists::id.eq(songs_artists::artist_id)))
        .inner_join(playbacks::table)
        .filter(playbacks::user_id.eq(user_id))
        .filter(artists::name.eq(artist))
        .order(sum(playbacks::count).desc().nulls_last())
        .limit(count.unwrap_or(50) as _)
        .get_results::<SongId3Db>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_top_songs_handler(
    State(database): State<Database>,
    req: GetTopSongsRequest,
) -> GetTopSongsJsonResponse {
    let pool = &database.pool;
    Ok(axum::Json(
        GetTopSongsBody {
            top_songs: TopSongs {
                song: stream::iter(
                    get_top_songs(pool, req.user_id, req.params.artist, req.params.count).await?,
                )
                .then(|v| async move { v.into(pool).await })
                .try_collect()
                .await?,
            },
        }
        .into(),
    ))
}
