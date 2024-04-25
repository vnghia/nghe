use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetArtistInfo2Params);
add_axum_response!(GetArtistInfo2Body);

pub async fn get_artist_info2(
    pool: &DatabasePool,
    artist_id: Uuid,
) -> Result<artists::LastfmInfo<'static>> {
    artists::table
        .filter(artists::id.eq(artist_id))
        .select(artists::LastfmInfo::as_select())
        .get_result(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_artist_info2_handler(
    State(database): State<Database>,
    req: GetArtistInfo2Request,
) -> GetArtistInfo2JsonResponse {
    Ok(axum::Json(
        GetArtistInfo2Body {
            artist_info2: get_artist_info2(&database.pool, req.params.id).await?.into(),
        }
        .into(),
    ))
}
