use std::borrow::Cow;

use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use o2o::o2o;
use typed_path::Utf8NativePath;
use uuid::Uuid;

use super::{Album, Artists, Genres};
use crate::database::Database;
use crate::orm::upsert::Upsert as _;
use crate::orm::{albums, songs};
use crate::scan::scanner;
use crate::{file, Error};

#[derive(Debug, o2o)]
#[ref_try_into(songs::Data<'a>, Error)]
#[cfg_attr(test, derive(PartialEq, Eq, fake::Dummy, Clone))]
pub struct Information<'a> {
    #[ref_into(songs::Data<'a>| song, (&~.song).try_into()?)]
    pub metadata: super::Metadata<'a>,
    #[map(~.try_into()?)]
    pub property: super::Property,
    #[map(~.into())]
    pub file: file::Property<super::Format>,
}

impl Information<'_> {
    pub async fn upsert_album(
        &self,
        database: &Database,
        foreign: albums::Foreign,
    ) -> Result<Uuid, Error> {
        self.metadata.album.upsert(database, foreign).await
    }

    pub async fn upsert_artists(
        &self,
        database: &Database,
        prefixes: &[impl AsRef<str>],
        song_id: Uuid,
    ) -> Result<(), Error> {
        self.metadata.artists.upsert(database, prefixes, song_id).await
    }

    pub async fn upsert_genres(&self, database: &Database, song_id: Uuid) -> Result<(), Error> {
        let genre_ids = self.metadata.genres.upsert(database).await?;
        Genres::upsert_song(database, song_id, &genre_ids).await
    }

    pub async fn upsert_cover_art(
        &self,
        database: &Database,
        dir: Option<&impl AsRef<Utf8NativePath>>,
    ) -> Result<Option<Uuid>, Error> {
        Ok(
            if let Some(ref picture) = self.metadata.picture
                && let Some(dir) = dir
            {
                Some(picture.upsert(database, dir).await?)
            } else {
                None
            },
        )
    }

    pub async fn upsert_song(
        &self,
        database: &Database,
        foreign: songs::Foreign,
        relative_path: impl Into<Cow<'_, str>>,
        id: impl Into<Option<Uuid>>,
    ) -> Result<Uuid, Error> {
        songs::Upsert { foreign, relative_path: relative_path.into(), data: self.try_into()? }
            .upsert(database, id)
            .await
    }

    pub async fn upsert(
        &self,
        database: &Database,
        config: &scanner::Config,
        foreign: albums::Foreign,
        relative_path: impl Into<Cow<'_, str>>,
        song_id: impl Into<Option<Uuid>>,
    ) -> Result<Uuid, Error> {
        let album_id = self.upsert_album(database, foreign).await?;
        let cover_art_id = self.upsert_cover_art(database, config.cover_art.dir.as_ref()).await?;
        let foreign = songs::Foreign { album_id, cover_art_id };

        let song_id = self.upsert_song(database, foreign, relative_path, song_id).await?;
        self.upsert_artists(database, &config.index.ignore_prefixes, song_id).await?;
        self.upsert_genres(database, song_id).await?;
        Ok(song_id)
    }

    pub async fn cleanup_one(
        database: &Database,
        started_at: time::OffsetDateTime,
        song_id: Uuid,
    ) -> Result<(), Error> {
        Artists::cleanup_one(database, started_at, song_id).await?;
        Genres::cleanup_one(database, started_at, song_id).await?;
        Ok(())
    }

    pub async fn cleanup(
        database: &Database,
        started_at: time::OffsetDateTime,
    ) -> Result<(), Error> {
        diesel::delete(songs::table)
            .filter(songs::scanned_at.lt(started_at))
            .execute(&mut database.get().await?)
            .await?;
        Album::cleanup(database).await?;
        Artists::cleanup(database).await?;
        Genres::cleanup(database).await?;
        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use crate::test::{mock, Information, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_information_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] update_information: bool,
    ) {
        let information = Information::builder().build();
        let id = information.upsert_mock(&mock, 0, None).await;
        let database_information = Information::query(&mock, id).await;
        assert_eq!(database_information, information);

        if update_information {
            let timestamp = crate::time::now().await;

            let update_information = Information::builder().build();
            let update_id = update_information.upsert_mock(&mock, 0, id).await;
            super::Information::cleanup_one(mock.database(), timestamp, id).await.unwrap();
            let database_update_information = Information::query(&mock, id).await;
            assert_eq!(update_id, id);
            assert_eq!(database_update_information, update_information);
        }
    }
}
