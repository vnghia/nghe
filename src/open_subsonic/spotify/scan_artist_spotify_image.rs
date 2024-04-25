use std::path::{Path, PathBuf};

use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use time::OffsetDateTime;
use uuid::Uuid;

use super::utils::search_and_upsert_artist_spotify_image;
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(ScanArtistSpotifyImageParams);
add_axum_response!(ScanArtistSpotifyImageBody);

#[tracing::instrument(skip_all)]
pub async fn scan_artist_spotify_image<P: AsRef<Path>>(
    pool: &DatabasePool,
    artist_art_dir: P,
    client: &rspotify::ClientCredsSpotify,
    artist_updated_at: Option<OffsetDateTime>,
) -> Result<()> {
    tracing::info!("Start scanning artist spotify image");

    let max_count = 100_usize;
    let mut current_offset = 0_usize;
    let mut upserted_artist_count = 0_usize;

    loop {
        // Can not filter by nullable `spotify_id` because it will change the offset.
        let artists = if let Some(artist_updated_at) = artist_updated_at {
            artists::table
                .filter(artists::updated_at.ge(artist_updated_at))
                .limit(max_count as i64)
                .offset(current_offset as i64)
                .order(artists::id)
                .select((artists::id, artists::name, artists::spotify_id))
                .get_results::<(Uuid, String, Option<String>)>(&mut pool.get().await?)
                .await?
        } else {
            artists::table
                .limit(max_count as i64)
                .offset(current_offset as i64)
                .order(artists::id)
                .select((artists::id, artists::name, artists::spotify_id))
                .get_results::<(Uuid, String, Option<String>)>(&mut pool.get().await?)
                .await?
        };

        if artists.is_empty() {
            break;
        } else {
            for (id, name, spotify_id) in &artists {
                if spotify_id.is_none() {
                    if let Err(e) = search_and_upsert_artist_spotify_image(
                        pool,
                        &artist_art_dir,
                        client,
                        *id,
                        name,
                    )
                    .await
                    {
                        tracing::error!(artist=name, upserting_artist_spotify_image=?e);
                    } else {
                        upserted_artist_count += 1;
                    }
                }
            }

            current_offset += artists.len();
            tracing::debug!(current_offset);
        }
    }
    tracing::info!(upserted_artist_count, "Finish scanning artist spotify image");

    Ok(())
}

pub async fn scan_artist_spotify_image_handler(
    State(database): State<Database>,
    Extension(artist_art_dir): Extension<Option<PathBuf>>,
    Extension(client): Extension<Option<rspotify::ClientCredsSpotify>>,
    req: ScanArtistSpotifyImageRequest,
) -> ScanArtistSpotifyImageJsonResponse {
    if let Some(client) = client
        && let Some(artist_art_dir) = artist_art_dir
    {
        tokio::task::spawn(async move {
            if let Err(e) = scan_artist_spotify_image(
                &database.pool,
                &artist_art_dir,
                &client,
                req.params.artist_updated_at,
            )
            .await
            {
                tracing::error!(scanning_artist_lastfm_info=?e);
            }
        });
    }
    Ok(axum::Json(ScanArtistSpotifyImageBody {}.into()))
}

#[cfg(all(test, spotify_env))]
mod tests {
    use time::macros::datetime;

    use super::*;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_scan_artist_spotify_image() {
        let infra = Infra::new().await;
        let artist_art_dir = infra.fs.art_config.artist_dir.as_ref().unwrap();
        let artist_id = upsert_artists(infra.pool(), &[], &["Micheal Learn To Rock".into()])
            .await
            .unwrap()
            .remove(0);

        scan_artist_spotify_image(
            infra.pool(),
            artist_art_dir,
            infra.spotify_client.as_ref().unwrap(),
            None,
        )
        .await
        .unwrap();

        let (cover_art_id, spotify_id) = artists::table
            .filter(artists::id.eq(artist_id))
            .select((artists::cover_art_id, artists::spotify_id))
            .get_result::<(Option<Uuid>, Option<String>)>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert!(cover_art_id.is_some());
        assert_eq!(spotify_id.unwrap(), "7zMVPOJPs5jgU8NorRxqJe");
    }

    #[tokio::test]
    async fn test_scan_artist_spotify_image_updated_at() {
        let infra = Infra::new().await;
        let artist_art_dir = infra.fs.art_config.artist_dir.as_ref().unwrap();
        let artist_id = upsert_artists(infra.pool(), &[], &["Micheal Learn To Rock".into()])
            .await
            .unwrap()
            .remove(0);

        scan_artist_spotify_image(
            infra.pool(),
            artist_art_dir,
            infra.spotify_client.as_ref().unwrap(),
            Some(datetime!(2000-01-01 0:00 UTC)),
        )
        .await
        .unwrap();

        let (cover_art_id, spotify_id) = artists::table
            .filter(artists::id.eq(artist_id))
            .select((artists::cover_art_id, artists::spotify_id))
            .get_result::<(Option<Uuid>, Option<String>)>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert!(cover_art_id.is_some());
        assert_eq!(spotify_id.unwrap(), "7zMVPOJPs5jgU8NorRxqJe");
    }
}
