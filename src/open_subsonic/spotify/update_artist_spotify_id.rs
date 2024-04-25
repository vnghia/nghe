use std::path::{Path, PathBuf};

use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use rspotify::clients::BaseClient;
use rspotify::model::ArtistId;
use uuid::Uuid;

use super::utils::upsert_artist_spotify_image;
use crate::{Database, DatabasePool, OSError};

add_common_validate!(UpdateArtistSpotifyIdParams);
add_axum_response!(UpdateArtistSpotifyIdBody);

async fn update_artist_spotify_id<P: AsRef<Path>>(
    pool: &DatabasePool,
    artist_art_dir: P,
    client: &rspotify::ClientCredsSpotify,
    id: Uuid,
    spotify_url: &str,
) -> Result<()> {
    client.refresh_token().await?;

    let artist = client
        .artist(ArtistId::from_id(
            spotify_url.strip_prefix("https://open.spotify.com/artist/").ok_or_else(|| {
                OSError::InvalidParameter(
                    "spotify url must starts with https://open.spotify.com/artist/".into(),
                )
            })?,
        )?)
        .await?;
    upsert_artist_spotify_image(pool, artist_art_dir, id, &artist).await?;

    Ok(())
}

pub async fn update_artist_spotify_id_handler(
    State(database): State<Database>,
    Extension(artist_art_dir): Extension<Option<PathBuf>>,
    Extension(client): Extension<Option<rspotify::ClientCredsSpotify>>,
    req: UpdateArtistSpotifyIdRequest,
) -> UpdateArtistSpotifyIdJsonResponse {
    if let Some(client) = client
        && let Some(artist_art_dir) = artist_art_dir
    {
        update_artist_spotify_id(
            &database.pool,
            artist_art_dir,
            &client,
            req.params.artist_id,
            &req.params.spotify_url,
        )
        .await?;
    }
    Ok(axum::Json(UpdateArtistSpotifyIdBody {}.into()))
}

#[cfg(all(test, spotify_env))]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;

    use super::*;
    use crate::models::*;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_update_artist_spotify_id() {
        let infra = Infra::new().await;
        let artist_art_dir = infra.fs.art_config.artist_dir.as_ref().unwrap();
        let artist_id =
            upsert_artists(infra.pool(), &[], &["artistwithnametoolongthatwontexist".into()])
                .await
                .unwrap()
                .remove(0);

        let (cover_art_id, spotify_id) = artists::table
            .filter(artists::id.eq(artist_id))
            .select((artists::cover_art_id, artists::spotify_id))
            .get_result::<(Option<Uuid>, Option<String>)>(&mut infra.pool().get().await.unwrap())
            .await
            .unwrap();
        assert!(cover_art_id.is_none());
        assert!(spotify_id.is_none());

        update_artist_spotify_id(
            infra.pool(),
            artist_art_dir,
            infra.spotify_client.as_ref().unwrap(),
            artist_id,
            "https://open.spotify.com/artist/3fMbdgg4jU18AjLCKBhRSm",
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
        assert_eq!(spotify_id.unwrap(), "3fMbdgg4jU18AjLCKBhRSm");
    }
}
