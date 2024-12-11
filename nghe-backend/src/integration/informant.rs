use std::borrow::Cow;

use diesel::dsl::{exists, not};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::media_annotation::update_artist_information::Request;
use rspotify::model::Id;
use typed_path::Utf8PlatformPath;
use uuid::Uuid;

use super::spotify;
use crate::database::Database;
use crate::file::picture;
use crate::orm::upsert::Update;
use crate::orm::{artist_informations, artists};
use crate::{config, Error};

const MAX_ITEM_PER_QUERY: i64 = 100;

#[derive(Clone)]
pub struct Informant {
    spotify: Option<spotify::Client>,
    reqwest: Option<reqwest::Client>,
}

impl Informant {
    pub async fn new(config: config::Integration) -> Self {
        let spotify = spotify::Client::new(config.spotify).await;
        let reqwest = if spotify.is_some() { Some(reqwest::Client::new()) } else { None };
        Self { spotify, reqwest }
    }

    pub fn is_enabled(&self) -> bool {
        self.spotify.is_some() || self.reqwest.is_some()
    }

    async fn upsert_artist_picture(
        &self,
        database: &Database,
        dir: Option<&impl AsRef<Utf8PlatformPath>>,
        source: Option<impl Into<Cow<'_, str>>>,
    ) -> Result<Option<Uuid>, Error> {
        Ok(
            if let Some(ref client) = self.reqwest
                && let Some(dir) = dir
                && let Some(source) = source
            {
                // TODO: Checking source before upserting.
                let picture = picture::Picture::fetch(client, source).await?;
                Some(picture.upsert(database, dir).await?)
            } else {
                None
            },
        )
    }

    async fn upsert_artist(
        &self,
        database: &Database,
        config: &config::CoverArt,
        id: Uuid,
        spotify: Option<&spotify::Artist>,
    ) -> Result<(), Error> {
        let spotify = if let Some(spotify) = spotify {
            let picture_id = self
                .upsert_artist_picture(database, config.dir.as_ref(), spotify.image_url.as_ref())
                .await?;
            artist_informations::Spotify {
                id: Some(spotify.id.id().into()),
                cover_art_id: picture_id,
            }
        } else {
            artist_informations::Spotify::default()
        };
        artist_informations::Data { spotify }.update(database, id).await
    }

    #[tracing::instrument(skip(self, database, config))]
    async fn search_and_upsert_artist(
        &self,
        database: &Database,
        config: &config::CoverArt,
        artist: &artists::Artist<'_>,
    ) -> Result<(), Error> {
        let id = artist.id;
        let spotify = if let Some(ref client) = self.spotify {
            client.search_artist(&artist.data.name).await?
        } else {
            None
        };

        self.upsert_artist(database, config, id, spotify.as_ref()).await
    }

    pub async fn search_and_upsert_artists(
        &self,
        database: &Database,
        config: &config::CoverArt,
    ) -> Result<(), Error> {
        if self.is_enabled() {
            loop {
                let artists =
                    query::artist_no_information().get_results(&mut database.get().await?).await?;
                if artists.is_empty() {
                    break;
                }
                for artist in artists {
                    self.search_and_upsert_artist(database, config, &artist).await?;
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
                id,
                if let Some(ref client) = self.spotify
                    && let Some(ref spotify_id) = request.spotify_id
                {
                    Some(client.fetch_artist(spotify_id).await?)
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
    use diesel::dsl::{auto_type, AsSelect};

    use super::*;

    #[auto_type]
    pub fn artist_no_information<'a>() -> _ {
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

#[cfg(all(test, spotify_env))]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_search_artist(
        #[future(awt)]
        #[with(0, 1, None, true)]
        mock: Mock,
    ) {
        let id_westlife = audio::Artist::from("Westlife").upsert_mock(&mock).await;
        let id_mltr = audio::Artist::from("Micheal Learns To Rock").upsert_mock(&mock).await;

        diesel::insert_into(artist_informations::table)
            .values(artist_informations::artist_id.eq(id_westlife))
            .execute(&mut mock.get().await)
            .await
            .unwrap();
        mock.informant
            .search_and_upsert_artists(mock.database(), &mock.config.cover_art)
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
    }
}
