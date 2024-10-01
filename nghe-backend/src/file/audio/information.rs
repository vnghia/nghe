use std::borrow::Cow;

use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use o2o::o2o;
use uuid::Uuid;

use super::{Artists, Genres};
use crate::database::Database;
use crate::orm::songs;
use crate::orm::upsert::Upsert as _;
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

impl<'a> Information<'a> {
    pub async fn upsert_album(
        &self,
        database: &Database,
        music_folder_id: Uuid,
    ) -> Result<Uuid, Error> {
        self.metadata.album.upsert(database, music_folder_id).await
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

    pub async fn upsert_song(
        &self,
        database: &Database,
        album_id: Uuid,
        relative_path: impl Into<Cow<'_, str>>,
        id: impl Into<Option<Uuid>>,
    ) -> Result<Uuid, Error> {
        songs::Upsert { album_id, relative_path: relative_path.into(), data: self.try_into()? }
            .upsert(database, id)
            .await
    }

    pub async fn upsert(
        &self,
        database: &Database,
        music_folder_id: Uuid,
        relative_path: impl Into<Cow<'_, str>>,
        prefixes: &[impl AsRef<str>],
        song_id: impl Into<Option<Uuid>>,
    ) -> Result<Uuid, Error> {
        let album_id = self.upsert_album(database, music_folder_id).await?;
        let song_id = self.upsert_song(database, album_id, relative_path, song_id).await?;
        self.upsert_artists(database, prefixes, song_id).await?;
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
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::Information;
    use crate::file::audio;
    use crate::orm::songs;
    use crate::test::Mock;

    impl Information<'static> {
        pub async fn query_data(mock: &Mock, id: Uuid) -> songs::Data<'static> {
            songs::table
                .filter(songs::id.eq(id))
                .select(songs::Data::as_select())
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
        }

        pub async fn query_upsert(mock: &Mock, id: Uuid) -> songs::Upsert<'static> {
            songs::table
                .filter(songs::id.eq(id))
                .select(songs::Upsert::as_select())
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
        }

        pub async fn query_path(mock: &Mock, id: Uuid) -> (String, Self) {
            let upsert = Self::query_upsert(mock, id).await;
            let album = audio::Album::query(mock, upsert.album_id).await;
            let artists = audio::Artists::query(mock, id).await;
            let genres = audio::Genres::query(mock, id).await;

            (
                upsert.relative_path.into_owned(),
                Self {
                    metadata: audio::Metadata {
                        song: upsert.data.song.try_into().unwrap(),
                        album,
                        artists,
                        genres,
                    },
                    property: upsert.data.property.try_into().unwrap(),
                    file: upsert.data.file.into(),
                },
            )
        }

        pub async fn query(mock: &Mock, id: Uuid) -> Self {
            Self::query_path(mock, id).await.1
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_information_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] update_information: bool,
    ) {
        let database = mock.database();
        let music_folder_id = mock.music_folder(0).await.music_folder.id;
        let relative_path: String = Faker.fake();
        let prefixes = &mock.config.index.ignore_prefixes;

        let information: Information = Faker.fake();
        let id = information
            .upsert(database, music_folder_id, &relative_path, prefixes, None)
            .await
            .unwrap();
        let database_information = Information::query(&mock, id).await;
        assert_eq!(database_information, information);

        if update_information {
            let timestamp = time::OffsetDateTime::now_utc();

            let update_information: Information = Faker.fake();
            let update_id = update_information
                .upsert(database, music_folder_id, &relative_path, prefixes, id)
                .await
                .unwrap();
            Information::cleanup_one(database, timestamp, id).await.unwrap();
            let database_update_information = Information::query(&mock, id).await;
            assert_eq!(update_id, id);
            assert_eq!(database_update_information, update_information);
        }
    }
}
