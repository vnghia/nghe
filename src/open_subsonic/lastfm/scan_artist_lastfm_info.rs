use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use time::OffsetDateTime;
use uuid::Uuid;

use super::utils::upsert_artist_lastfm_info;
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(ScanArtistLastfmInfoParams);
add_axum_response!(ScanArtistLastfmInfoBody);

#[tracing::instrument(skip_all)]
pub async fn scan_artist_lastfm_info(
    pool: &DatabasePool,
    client: &lastfm_client::Client,
    artist_updated_at: Option<OffsetDateTime>,
) -> Result<()> {
    let max_count = 100_usize;
    let mut current_offset = 0_usize;
    loop {
        let artists = if let Some(artist_updated_at) = artist_updated_at {
            artists::table
                .filter(
                    artists::updated_at.ge(artist_updated_at).and(artists::lastfm_url.is_null()),
                )
                .limit(max_count as i64)
                .offset(current_offset as i64)
                .select((artists::id, artists::name, artists::mbz_id))
                .get_results::<(Uuid, String, Option<Uuid>)>(&mut pool.get().await?)
                .await?
        } else {
            artists::table
                .filter(artists::lastfm_url.is_null())
                .limit(max_count as i64)
                .offset(current_offset as i64)
                .select((artists::id, artists::name, artists::mbz_id))
                .get_results::<(Uuid, String, Option<Uuid>)>(&mut pool.get().await?)
                .await?
        };

        for (id, name, mbz_id) in &artists {
            if let Err(e) = upsert_artist_lastfm_info(pool, client, *id, name, *mbz_id).await {
                tracing::error!(artist=name, upserting_artist_lastfm_info=?e);
            }
        }
        if artists.len() < max_count {
            break;
        } else {
            current_offset += max_count;
            tracing::debug!(current_offset = current_offset);
        }
    }

    Ok(())
}

pub async fn scan_artist_lastfm_info_handler(
    State(database): State<Database>,
    Extension(client): Extension<Option<lastfm_client::Client>>,
    req: ScanArtistLastfmInfoRequest,
) -> ScanArtistLastfmInfoJsonResponse {
    if let Some(client) = client {
        tokio::task::spawn(async move {
            if let Err(e) =
                scan_artist_lastfm_info(&database.pool, &client, req.params.artist_updated_at).await
            {
                tracing::error!(scanning_artist_lastfm_info=?e);
            }
        });
    }
    Ok(axum::Json(ScanArtistLastfmInfoBody {}.into()))
}
