use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use lastfm_client::artist;
use uuid::Uuid;

use crate::models::*;
use crate::DatabasePool;

pub async fn upsert_artist_lastfm_info(
    pool: &DatabasePool,
    client: &lastfm_client::Client,
    id: Uuid,
    name: &str,
    mbz_id: Option<Uuid>,
) -> Result<()> {
    let (artist, mbid) = if let Some(mbz_id) = mbz_id {
        (Some(name.into()), Some(mbz_id))
    } else {
        let mut artists = client
            .send::<artist::search::Response>(&artist::search::Params {
                artist: name.into(),
                limit: Some(1),
                page: Some(1),
            })
            .await?
            .results
            .artist_matches
            .artist;
        if artists.is_empty() {
            return Ok(());
        } else {
            let artist = artists.remove(0);
            (Some(artist.name.into()), artist.mbid)
        }
    };

    let artist = client
        .send::<artist::get_info::Response>(&artist::get_info::Params { artist, mbid })
        .await?
        .artist;

    diesel::update(artists::table)
        .filter(artists::id.eq(id))
        .set(artists::LastfmInfo {
            lastfm_url: Some(artist.artist.url.into()),
            lastfm_mbz_id: artist.artist.mbid,
            lastfm_biography: artist.bio.summary.map(|s| s.into()),
        })
        .execute(&mut pool.get().await?)
        .await?;

    Ok(())
}

#[cfg(all(test, lastfm_env))]
mod tests {
    use super::*;
    use crate::open_subsonic::browsing::test::get_artist_info2;
    use crate::open_subsonic::scan::test::upsert_artists;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_upsert_artist() {
        let infra = Infra::new().await;
        let artist_id =
            upsert_artists(infra.pool(), &[], &["cher".into()]).await.unwrap().remove(0);
        upsert_artist_lastfm_info(
            infra.pool(),
            infra.lastfm_client.as_ref().unwrap(),
            artist_id,
            "cher",
            None,
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

    #[tokio::test]
    async fn test_upsert_artist_non_existent() {
        let infra = Infra::new().await;
        let artist_id =
            upsert_artists(infra.pool(), &[], &["artistwithnametoolongthatwontexist".into()])
                .await
                .unwrap()
                .remove(0);
        upsert_artist_lastfm_info(
            infra.pool(),
            infra.lastfm_client.as_ref().unwrap(),
            artist_id,
            "artistwithnametoolongthatwontexist",
            None,
        )
        .await
        .unwrap();
        let artist_info = get_artist_info2(infra.pool(), artist_id).await.unwrap();
        assert!(artist_info.lastfm_mbz_id.is_none());
        assert!(artist_info.lastfm_url.is_none());
        assert!(artist_info.lastfm_biography.is_none());
    }
}
