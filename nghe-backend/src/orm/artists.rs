use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::artists::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    pub index: Cow<'a, str>,
    #[diesel(embed)]
    pub data: Data<'a>,
}

mod upsert {
    use diesel::{DecoratableTarget, ExpressionMethods};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{artists, Upsert};
    use crate::database::Database;
    use crate::Error;

    impl<'a> crate::orm::upsert::Insert for Upsert<'a> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            if self.data.mbz_id.is_some() {
                diesel::insert_into(artists::table)
                    .values(self)
                    .on_conflict(artists::mbz_id)
                    .do_update()
                    .set((self, artists::scanned_at.eq(time::OffsetDateTime::now_utc())))
                    .returning(artists::id)
                    .get_result(&mut database.get().await?)
                    .await
            } else {
                diesel::insert_into(artists::table)
                    .values(self)
                    .on_conflict(artists::name)
                    .filter_target(artists::mbz_id.is_null())
                    .do_update()
                    .set((self, artists::scanned_at.eq(time::OffsetDateTime::now_utc())))
                    .returning(artists::id)
                    .get_result(&mut database.get().await?)
                    .await
            }
            .map_err(Error::from)
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::{fixture, rstest};
    use uuid::Uuid;

    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_artist_upsert_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] mbz_id: bool,
        #[values(true, false)] update_artist: bool,
    ) {
        let mbz_id = if mbz_id { Some(Faker.fake()) } else { None };
        let artist = audio::Artist { mbz_id, ..Faker.fake() };
        let id = artist.upsert(mock.database(), &[""]).await.unwrap();
        let database_artist = audio::Artist::query(&mock, id).await;
        assert_eq!(database_artist, artist);

        if update_artist {
            let update_artist = audio::Artist { mbz_id, ..Faker.fake() };
            let update_id = update_artist.upsert(mock.database(), &[""]).await.unwrap();
            let database_update_artist = audio::Artist::query(&mock, id).await;
            if mbz_id.is_some() {
                assert_eq!(id, update_id);
                assert_eq!(database_update_artist, update_artist);
            } else {
                // This will always insert a new row to the database
                // since there is nothing to identify an old artist.
                assert_ne!(id, update_id);
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_artist_upsert_no_mbz_id(#[future(awt)] mock: Mock) {
        // We want to make sure that insert the same artist with no mbz_id
        // twice does not result in any error.
        let artist = audio::Artist { mbz_id: None, ..Faker.fake() };
        let id = artist.upsert(mock.database(), &[""]).await.unwrap();
        let update_id = artist.upsert(mock.database(), &[""]).await.unwrap();
        assert_eq!(update_id, id);
    }

    #[fixture]
    async fn mock_with_song(#[future(awt)] mock: Mock) -> (Mock, Uuid) {
        let information: audio::Information = Faker.fake();
        let album_id = information.metadata.album.upsert_mock(&mock, 0).await;
        let song_id = information
            .upsert(mock.database(), album_id, Faker.fake::<String>(), None)
            .await
            .unwrap();
        (mock, song_id)
    }

    #[rstest]
    #[tokio::test]
    async fn test_artists_upsert(
        #[future(awt)] mock: Mock,
        #[values(true, false)] compilation: bool,
    ) {
        let information: audio::Information = Faker.fake();
        let album_id = information.metadata.album.upsert_mock(&mock, 0).await;
        let song_id = information
            .upsert(mock.database(), album_id, Faker.fake::<String>(), None)
            .await
            .unwrap();

        let artists = audio::Artists { compilation, ..Faker.fake() };
        artists.upsert(mock.database(), &[""], song_id).await.unwrap();

        let database_artists = audio::Artists::query(&mock, song_id).await;
        assert_eq!(database_artists, artists);
    }
}
