use std::borrow::Borrow;

use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use rspotify::clients::BaseClient;
use rspotify::model::{FullArtist, SearchResult, SearchType};
use uuid::Uuid;

use crate::models::*;
use crate::open_subsonic::cover_art::upsert_cover_art_from_url;
use crate::utils::fs::LocalPath;
use crate::DatabasePool;

pub async fn upsert_artist_spotify_image<P: AsRef<LocalPath>>(
    pool: &DatabasePool,
    artist_art_dir: P,
    id: Uuid,
    artist: &FullArtist,
) -> Result<()> {
    let artist_id = &artist.id;
    if artist.images.is_empty() {
        diesel::update(artists::table)
            .filter(artists::id.eq(id))
            .set(artists::spotify_id.eq::<&str>(artist_id.borrow()))
            .execute(&mut pool.get().await?)
            .await?;
    } else {
        diesel::update(artists::table)
            .filter(artists::id.eq(id))
            .set((
                artists::cover_art_id.eq(upsert_cover_art_from_url(
                    pool,
                    artist_art_dir,
                    &artist.images[0].url,
                )
                .await?),
                artists::spotify_id.eq::<&str>(artist_id.borrow()),
            ))
            .execute(&mut pool.get().await?)
            .await?;
    };
    Ok(())
}

pub async fn search_and_upsert_artist_spotify_image<P: AsRef<LocalPath>>(
    pool: &DatabasePool,
    artist_art_dir: P,
    client: &rspotify::ClientCredsSpotify,
    id: Uuid,
    name: &str,
) -> Result<()> {
    client.refresh_token().await?;

    if let SearchResult::Artists(artists) =
        client.search(name, SearchType::Artist, None, None, None, None).await?
    {
        let artists = artists.items;
        if artists.is_empty() {
            return Ok(());
        }
        upsert_artist_spotify_image(pool, artist_art_dir, id, &artists[0]).await?;
    }

    Ok(())
}

#[cfg(all(test, spotify_env))]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;

    use super::*;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_search_and_upsert_artist_spotify_image() {
        let infra = Infra::new().await;
        let artist_art_dir = infra.fs.art_config.artist_dir.as_ref().unwrap();
        let artist_id = upsert_artists(infra.pool(), &[], &["Micheal Learn To Rock".into()])
            .await
            .unwrap()
            .remove(0);

        search_and_upsert_artist_spotify_image(
            infra.pool(),
            artist_art_dir,
            infra.spotify_client.as_ref().unwrap(),
            artist_id,
            "Micheal Learn To Rock",
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
