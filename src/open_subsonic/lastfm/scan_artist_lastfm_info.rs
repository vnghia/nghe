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
    tracing::info!("Start scanning artist lastfm info");

    let max_count = 100_usize;
    let mut current_offset = 0_usize;
    let mut upserted_artist_count = 0_usize;

    loop {
        let artists = if let Some(artist_updated_at) = artist_updated_at {
            artists::table
                .filter(
                    artists::updated_at.ge(artist_updated_at).and(artists::lastfm_url.is_null()),
                )
                .limit(max_count as i64)
                .offset(current_offset as i64)
                .order(artists::id)
                .select((artists::id, artists::name, artists::mbz_id))
                .get_results::<(Uuid, String, Option<Uuid>)>(&mut pool.get().await?)
                .await?
        } else {
            artists::table
                .filter(artists::lastfm_url.is_null())
                .limit(max_count as i64)
                .offset(current_offset as i64)
                .order(artists::id)
                .select((artists::id, artists::name, artists::mbz_id))
                .get_results::<(Uuid, String, Option<Uuid>)>(&mut pool.get().await?)
                .await?
        };

        for (id, name, mbz_id) in &artists {
            if let Err(e) = upsert_artist_lastfm_info(pool, client, *id, name, *mbz_id).await {
                tracing::error!(artist=name, upserting_artist_lastfm_info=?e);
            } else {
                upserted_artist_count += 1;
            }
        }
        if artists.len() < max_count {
            break;
        } else {
            current_offset += max_count;
            tracing::debug!(current_offset);
        }
    }
    tracing::info!(upserted_artist_count, "Finish scanning artist lastfm info");

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

#[cfg(all(test, lastfm_env))]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::open_subsonic::browsing::test::get_artist_info2;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_scan_artist_lastfm_info() {
        let infra = Infra::new().await;
        let artist_id =
            upsert_artists(infra.pool(), &[], &["cher".into()]).await.unwrap().remove(0);
        scan_artist_lastfm_info(infra.pool(), infra.lastfm_client.as_ref().unwrap(), None)
            .await
            .unwrap();
        let artist_info = get_artist_info2(infra.pool(), artist_id).await.unwrap();
        assert_eq!(
            artist_info.lastfm_mbz_id.unwrap(),
            Uuid::parse_str("bfcc6d75-a6a5-4bc6-8282-47aec8531818").unwrap()
        );
        assert_eq!(artist_info.lastfm_url.unwrap(), "https://www.last.fm/music/Cher");
        assert!(artist_info.lastfm_biography.is_some());
    }

    #[tokio::test]
    async fn test_scan_artist_lastfm_info_updated_at() {
        let infra = Infra::new().await;
        let artist_id =
            upsert_artists(infra.pool(), &[], &["cher".into()]).await.unwrap().remove(0);
        scan_artist_lastfm_info(
            infra.pool(),
            infra.lastfm_client.as_ref().unwrap(),
            Some(datetime!(2000-01-01 0:00 UTC)),
        )
        .await
        .unwrap();
        let artist_info = get_artist_info2(infra.pool(), artist_id).await.unwrap();
        assert_eq!(
            artist_info.lastfm_mbz_id.unwrap(),
            Uuid::parse_str("bfcc6d75-a6a5-4bc6-8282-47aec8531818").unwrap()
        );
        assert_eq!(artist_info.lastfm_url.unwrap(), "https://www.last.fm/music/Cher");
        assert!(artist_info.lastfm_biography.is_some());
    }
}
