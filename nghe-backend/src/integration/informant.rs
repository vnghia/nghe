use std::borrow::Cow;

use diesel::dsl::{exists, not};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use rspotify::model::Id;
use typed_path::Utf8NativePath;
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
        let reqwest = if spotify.is_none() { Some(reqwest::Client::new()) } else { None };
        Self { spotify, reqwest }
    }

    async fn upsert_artist_picture(
        &self,
        database: &Database,
        dir: Option<&impl AsRef<Utf8NativePath>>,
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

    #[tracing::instrument(skip(self, database, config, id), ret(level = "trace"))]
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
        artist_informations::Upsert { spotify }.update(database, id).await
    }

    #[tracing::instrument(skip(self, database, config))]
    async fn search_and_upsert_artist(
        &self,
        database: &Database,
        config: &config::CoverArt,
        artist: &artists::Artist<'_>,
    ) -> Result<(), Error> {
        let id = artist.id;
        self.upsert_artist(
            database,
            config,
            id,
            if let Some(ref client) = self.spotify {
                client.search_artist(&artist.data.name).await?
            } else {
                None
            }
            .as_ref(),
        )
        .await
    }

    pub async fn search_and_upsert_artists(
        &self,
        database: &Database,
        config: &config::CoverArt,
    ) -> Result<(), Error> {
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
