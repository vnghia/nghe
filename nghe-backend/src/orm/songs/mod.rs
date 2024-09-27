use std::borrow::Cow;
use std::str::FromStr;

use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel_derives::AsChangeset;
use uuid::Uuid;

use crate::file::audio;
pub use crate::schema::songs::{self, *};

pub mod date;
pub mod name_date_mbz;
pub mod position;
mod property;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Song<'a> {
    #[diesel(embed)]
    pub main: name_date_mbz::NameDateMbz<'a>,
    #[diesel(embed)]
    pub track_disc: position::TrackDisc,
    pub languages: Vec<Option<Cow<'a, str>>>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    #[diesel(embed)]
    pub song: Song<'a>,
    #[diesel(embed)]
    pub property: property::Property,
    #[diesel(embed)]
    pub file: property::File,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    pub album_id: Uuid,
    pub relative_path: Cow<'a, str>,
    #[diesel(embed)]
    pub data: Data<'a>,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{songs, Upsert};
    use crate::database::Database;
    use crate::Error;

    impl<'a> crate::orm::upsert::Insert for Upsert<'a> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            diesel::insert_into(songs::table)
                .values(self)
                .returning(songs::id)
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }
    }

    impl<'a> crate::orm::upsert::Update for Upsert<'a> {
        async fn update(&self, database: &Database, id: Uuid) -> Result<(), Error> {
            diesel::update(songs::table)
                .filter(songs::id.eq(id))
                .set(self)
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }

    impl<'a> crate::orm::upsert::Upsert for Upsert<'a> {}
}

impl ToSql<Text, super::Type> for audio::Format {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, super::Type>) -> serialize::Result {
        <str as ToSql<Text, super::Type>>::to_sql(self.as_ref(), out)
    }
}

impl FromSql<Text, super::Type> for audio::Format {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        Ok(audio::Format::from_str(core::str::from_utf8(bytes.as_bytes())?)?)
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_song_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(None, Some(Faker.fake()))] update_song: Option<audio::Information<'static>>,
    ) {
        use crate::file;

        let song: audio::Information = Faker.fake();
        let album_id = song.metadata.album.upsert_mock(&mock, 0).await;
        let id = song
            .upsert_song(mock.database(), album_id, Faker.fake::<String>(), None)
            .await
            .unwrap();

        let database_data = audio::Information::query_data(&mock, id).await;
        let database_song: audio::Song = database_data.song.try_into().unwrap();
        let database_property: audio::Property = database_data.property.try_into().unwrap();
        let database_file: file::Property<_> = database_data.file.into();
        assert_eq!(database_song, song.metadata.song);
        assert_eq!(database_property, song.property);
        assert_eq!(database_file, song.file);

        if let Some(update_song) = update_song {
            let update_id = update_song
                .upsert_song(mock.database(), album_id, Faker.fake::<String>(), id)
                .await
                .unwrap();

            let update_database_data = audio::Information::query_data(&mock, update_id).await;
            let update_database_song: audio::Song = update_database_data.song.try_into().unwrap();
            let update_database_property: audio::Property =
                update_database_data.property.try_into().unwrap();
            let update_database_file: file::Property<_> = update_database_data.file.into();
            assert_eq!(update_database_song, update_song.metadata.song);
            assert_eq!(update_database_property, update_song.property);
            assert_eq!(update_database_file, update_song.file);
        }
    }
}
