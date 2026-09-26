use std::borrow::Cow;

use diesel::dsl::{exists, not};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::media_annotation::update_artist_information::Request;
use rspotify::model::Id;
use typed_path::Utf8PlatformPath;
use uuid::Uuid;

use super::{lastfm, spotify};
use crate::database::Database;
use crate::file::image;
use crate::orm::upsert::Update;
use crate::orm::{artist_informations, artists};
use crate::{Error, config};

const MAX_ITEM_PER_QUERY: i64 = 100;

#[derive(Clone)]
pub struct Informant {
    reqwest: reqwest::Client,
    spotify: Option<spotify::Client>,
    lastfm: Option<lastfm::Client>,
}

impl Informant {
    pub async fn new(config: config::Integration) -> Self {
        let reqwest = reqwest::Client::new();
        let spotify = spotify::Client::new(config.spotify).await;
        let lastfm = lastfm::Client::new(reqwest.clone(), config.lastfm);
        Self { reqwest, spotify, lastfm }
    }

    pub fn is_enabled(&self) -> bool {
        self.spotify.is_some() || self.lastfm.is_some()
    }

    async fn upsert_artist_image(
        &self,
        database: &Database,
        full: bool,
        dir: Option<&impl AsRef<Utf8PlatformPath>>,
        url: Option<impl AsRef<str>>,
    ) -> Result<Option<Uuid>, Error> {
        Ok(
            if let Some(dir) = dir
                && let Some(url) = url.as_ref()
            {
                if !full && let Some(image_id) = image::Image::query_source(database, url).await? {
                    Some(image_id)
                } else {
                    let image = image::Image::fetch(&self.reqwest, url).await?;
                    Some(image.upsert(database, dir, Some(url)).await?)
                }
            } else {
                None
            },
        )
    }

    async fn upsert_artist(
        &self,
        database: &Database,
        config: &config::CoverArt,
        full: bool,
        id: Uuid,
        spotify: Option<&spotify::Artist>,
        lastfm: Option<&lastfm::model::artist::Full>,
    ) -> Result<(), Error> {
        let spotify = if let Some(spotify) = spotify {
            let image_id = self
                .upsert_artist_image(
                    database,
                    full,
                    config.dir.as_ref(),
                    spotify.image_url.as_ref(),
                )
                .await?;
            artist_informations::Spotify {
                id: Some(spotify.id.id().into()),
                cover_art_id: image_id,
            }
        } else {
            artist_informations::Spotify::default()
        };
        let lastfm = lastfm
            .map(|lastfm| artist_informations::Lastfm {
                url: Some(lastfm.short.url.as_str().into()),
                mbz_id: lastfm.short.mbid,
                biography: lastfm.bio.summary.as_deref().map(Cow::Borrowed),
            })
            .unwrap_or_default();
        artist_informations::Data { spotify, lastfm }.update(database, id).await
    }

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip(self, database, config)))]
    async fn search_and_upsert_artist(
        &self,
        database: &Database,
        config: &config::CoverArt,
        full: bool,
        artist: &artists::Artist<'_>,
    ) -> Result<(), Error> {
        let id = artist.id;
        let spotify = if let Some(ref client) = self.spotify {
            client.search_artist(&artist.data.name).await?
        } else {
            None
        };
        let lastfm = if let Some(ref client) = self.lastfm {
            client.search_and_fetch_artist(&artist.data.name, artist.data.mbz_id).await?
        } else {
            None
        };

        self.upsert_artist(database, config, full, id, spotify.as_ref(), lastfm.as_ref()).await
    }

    pub async fn search_and_upsert_artists(
        &self,
        database: &Database,
        config: &config::CoverArt,
        full: bool,
    ) -> Result<(), Error> {
        if self.is_enabled() {
            let mut offset = 0;
            loop {
                let artists = if full {
                    query::artists_full(offset).get_results(&mut database.get().await?).await?
                } else {
                    query::artists_no_information().get_results(&mut database.get().await?).await?
                };

                if artists.is_empty() {
                    break;
                }
                let artist_len: i64 = artists.len().try_into()?;
                offset += artist_len;

                for artist in artists {
                    self.search_and_upsert_artist(database, config, full, &artist).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn fetch_and_upsert_artist(
        &self,
        database: &Database,
        config: &config::CoverArt,
        request: &Request,
    ) -> Result<(), Error> {
        if self.is_enabled() {
            let id = request.artist_id;
            self.upsert_artist(
                database,
                config,
                true,
                id,
                if let Some(ref client) = self.spotify
                    && let Some(ref spotify_id) = request.spotify_id
                {
                    Some(client.fetch_artist(spotify_id).await?)
                } else {
                    None
                }
                .as_ref(),
                if let Some(ref client) = self.lastfm
                    && let Some(ref lastfm_name) = request.lastfm_name
                {
                    client.search_and_fetch_artist(lastfm_name, None).await?
                } else {
                    None
                }
                .as_ref(),
            )
            .await?;
        }
        Ok(())
    }
}

mod query {
    use diesel::dsl::{AsSelect, auto_type};

    use super::*;

    #[auto_type]
    pub fn artists_full<'a>(offset: i64) -> _ {
        let limit: i64 = MAX_ITEM_PER_QUERY;
        let artist: AsSelect<artists::Artist<'a>, crate::orm::Type> =
            artists::Artist::<'a>::as_select();
        artists::table.select(artist).limit(limit).offset(offset).order_by(artists::id)
    }

    #[auto_type]
    pub fn artists_no_information<'a>() -> _ {
        let limit: i64 = MAX_ITEM_PER_QUERY;
        let artist: AsSelect<artists::Artist<'a>, crate::orm::Type> =
            artists::Artist::<'a>::as_select();
        artists::table
            .filter(not(exists(
                artist_informations::table.filter(artist_informations::artist_id.eq(artists::id)),
            )))
            .select(artist)
            .limit(limit)
    }
}

#[cfg(all(test, spotify_env, lastfm_env))]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::file;
    use crate::file::audio;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_search_artist(
        #[future(awt)]
        #[with(0, 1, None, true)]
        mock: Mock,
        #[values(true, false)] full: bool,
    ) {
        let id_westlife = audio::Artist::from("Westlife").upsert_mock(&mock).await;
        let id_mltr = audio::Artist::from("Micheal Learns To Rock").upsert_mock(&mock).await;

        diesel::insert_into(artist_informations::table)
            .values(artist_informations::artist_id.eq(id_westlife))
            .execute(&mut mock.get().await)
            .await
            .unwrap();
        mock.informant
            .search_and_upsert_artists(mock.database(), &mock.config.cover_art, full)
            .await
            .unwrap();

        let westlife_data = artist_informations::table
            .filter(artist_informations::artist_id.eq(id_westlife))
            .select(artist_informations::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap();
        assert_eq!(westlife_data.spotify.id.is_some(), full);
        assert_eq!(westlife_data.spotify.cover_art_id.is_some(), full);
        assert_eq!(westlife_data.lastfm.url.is_some(), full);
        assert_eq!(westlife_data.lastfm.mbz_id.is_some(), full);
        assert_eq!(westlife_data.lastfm.biography.is_some(), full);

        let mltr_data = artist_informations::table
            .filter(artist_informations::artist_id.eq(id_mltr))
            .select(artist_informations::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap();
        assert!(mltr_data.spotify.id.is_some());
        assert!(mltr_data.spotify.cover_art_id.is_some());
        assert!(mltr_data.lastfm.url.is_some());
        assert!(mltr_data.lastfm.mbz_id.is_some());
        assert!(mltr_data.lastfm.biography.is_some());
    }

    #[rstest]
    #[tokio::test]
    async fn test_upsert_artist_image(
        #[future(awt)]
        #[with(0, 1, None, true)]
        mock: Mock,
        #[values(true, false)] full: bool,
    ) {
        let id = audio::Artist::from("Micheal Learns To Rock").upsert_mock(&mock).await;
        let config = &mock.config.cover_art;
        mock.informant.search_and_upsert_artists(mock.database(), config, false).await.unwrap();

        let data = artist_informations::table
            .filter(artist_informations::artist_id.eq(id))
            .select(artist_informations::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap();

        let cover_art_id = data.spotify.cover_art_id.unwrap();
        let image_path = file::Property::query_cover_art(mock.database(), cover_art_id)
            .await
            .unwrap()
            .image_path(config.dir.as_ref().unwrap());
        assert!(tokio::fs::try_exists(&image_path).await.unwrap());
        tokio::fs::remove_file(&image_path).await.unwrap();

        mock.informant.search_and_upsert_artists(mock.database(), config, full).await.unwrap();
        assert_eq!(tokio::fs::try_exists(&image_path).await.unwrap(), full);
    }

    #[rstest]
    #[tokio::test]
    async fn test_fetch_artist(
        #[future(awt)]
        #[with(0, 1, None, true)]
        mock: Mock,
    ) {
        let id_mltr = audio::Artist::from("Micheal Learns To Rock").upsert_mock(&mock).await;

        mock.informant
            .fetch_and_upsert_artist(
                mock.database(),
                &mock.config.cover_art,
                &Request {
                    artist_id: id_mltr,
                    spotify_id: Some("7zMVPOJPs5jgU8NorRxqJe".to_owned()),
                    lastfm_name: Some("Michael Learns to Rock".to_owned()),
                },
            )
            .await
            .unwrap();

        let mltr_data = artist_informations::table
            .filter(artist_informations::artist_id.eq(id_mltr))
            .select(artist_informations::Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap();
        assert!(mltr_data.spotify.id.is_some());
        assert!(mltr_data.spotify.cover_art_id.is_some());
        assert!(mltr_data.lastfm.url.is_some());
        assert!(mltr_data.lastfm.mbz_id.is_some());
        assert!(mltr_data.lastfm.biography.is_some());
    }
}
