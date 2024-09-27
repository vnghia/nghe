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
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use fake::{Fake, Faker};
    use futures_lite::{stream, StreamExt};
    use rstest::{fixture, rstest};
    use uuid::Uuid;

    use super::{artists, Data};
    use crate::file::audio;
    use crate::orm::{songs_album_artists, songs_artists};
    use crate::test::{mock, Mock};

    async fn select_artist(mock: &Mock, id: Uuid) -> audio::Artist<'static> {
        artists::table
            .filter(artists::id.eq(id))
            .select(Data::as_select())
            .get_result(&mut mock.get().await)
            .await
            .unwrap()
            .into()
    }

    #[rstest]
    #[case(false, false)]
    #[case(true, false)]
    #[case(false, true)]
    #[case(true, true)]
    #[tokio::test]
    async fn test_artist_upsert_roundtrip(
        #[future(awt)] mock: Mock,
        #[case] mbz_id: bool,
        #[case] update_artist: bool,
    ) {
        let mbz_id = if mbz_id { Some(Faker.fake()) } else { None };
        let artist = audio::Artist { mbz_id, ..Faker.fake() };
        let id = artist.upsert(mock.database(), &[""]).await.unwrap();
        let database_artist = select_artist(&mock, id).await;
        assert_eq!(database_artist, artist);

        if update_artist {
            let update_artist = audio::Artist { mbz_id, ..Faker.fake() };
            let update_id = update_artist.upsert(mock.database(), &[""]).await.unwrap();
            let database_update_artist = select_artist(&mock, id).await;
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

    async fn select_song_artists(mock: &Mock, song_id: Uuid) -> Vec<audio::Artist<'static>> {
        let ids: Vec<Uuid> = songs_artists::table
            .filter(songs_artists::song_id.eq(song_id))
            .select(songs_artists::artist_id)
            .order_by(songs_artists::upserted_at)
            .get_results(&mut mock.get().await)
            .await
            .unwrap();
        stream::iter(ids).then(async |id| select_artist(mock, id).await).collect().await
    }

    async fn select_song_album_artists(
        mock: &Mock,
        song_id: Uuid,
    ) -> (Vec<audio::Artist<'static>>, bool) {
        let ids_compilations = songs_album_artists::table
            .filter(songs_album_artists::song_id.eq(song_id))
            .select((songs_album_artists::album_artist_id, songs_album_artists::compilation))
            .order_by(songs_album_artists::upserted_at)
            .get_results::<(Uuid, bool)>(&mut mock.get().await)
            .await
            .unwrap();
        let artists: Vec<_> = stream::iter(&ids_compilations)
            .copied()
            .filter_map(|(id, compilation)| if compilation { None } else { Some(id) })
            .then(async |id| select_artist(mock, id).await)
            .collect()
            .await;
        // If there is any compliation, it will be filtered out and make the size of two vectors not
        // equal. On the other hand, two same size vectors can mean either there isn't any
        // compilation or the song artists are the same as the album artists or there isn't any
        // album artist (which then be filled with song artists).
        let compilation = ids_compilations.len() != artists.len();
        (artists, compilation)
    }

    #[rstest]
    #[tokio::test]
    async fn test_artists_upsert(#[future(awt)] mock_with_song: (Mock, Uuid)) {
        let (mock, song_id) = mock_with_song;
        let artists: audio::Artists = Faker.fake();
        artists.upsert(mock.database(), &[""], song_id).await.unwrap();

        let database_song = select_song_artists(&mock, song_id).await;
        let (database_album, compilation) = select_song_album_artists(&mock, song_id).await;
        let database_artists =
            audio::Artists::new(database_song, database_album, compilation).unwrap();
        assert_eq!(database_artists, artists);
    }
}
