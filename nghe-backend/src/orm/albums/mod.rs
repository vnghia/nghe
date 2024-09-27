use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::albums::{self, *};

pub mod date;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    pub name: Cow<'a, str>,
    #[diesel(embed)]
    pub date: date::Date,
    #[diesel(embed)]
    pub release_date: date::Release,
    #[diesel(embed)]
    pub original_release_date: date::OriginalRelease,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    pub music_folder_id: Uuid,
    #[diesel(embed)]
    pub data: Data<'a>,
}

mod upsert {
    use diesel::{DecoratableTarget, ExpressionMethods};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{albums, Upsert};
    use crate::database::Database;
    use crate::Error;

    impl<'a> crate::orm::upsert::Insert for Upsert<'a> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            if self.data.mbz_id.is_some() {
                diesel::insert_into(albums::table)
                    .values(self)
                    .on_conflict((albums::music_folder_id, albums::mbz_id))
                    .do_update()
                    .set((self, albums::scanned_at.eq(time::OffsetDateTime::now_utc())))
                    .returning(albums::id)
                    .get_result(&mut database.get().await?)
                    .await
            } else {
                diesel::insert_into(albums::table)
                    .values(self)
                    .on_conflict((
                        albums::music_folder_id,
                        albums::name,
                        albums::year,
                        albums::month,
                        albums::day,
                        albums::release_year,
                        albums::release_month,
                        albums::release_day,
                        albums::original_release_year,
                        albums::original_release_month,
                        albums::original_release_day,
                    ))
                    .filter_target(albums::mbz_id.is_null())
                    .do_update()
                    .set(albums::scanned_at.eq(time::OffsetDateTime::now_utc()))
                    .returning(albums::id)
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
    use rstest::rstest;

    use crate::file::audio;
    use crate::test::{mock, Mock};

    #[rstest]
    #[case(false, false)]
    #[case(true, false)]
    #[case(false, true)]
    #[case(true, true)]
    #[tokio::test]
    async fn test_album_upsert_roundtrip(
        #[future(awt)] mock: Mock,
        #[case] mbz_id: bool,
        #[case] update_album: bool,
    ) {
        let mbz_id = if mbz_id { Some(Faker.fake()) } else { None };
        let album = audio::NameDateMbz { mbz_id, ..Faker.fake() };
        let id = album.upsert_mock(&mock, 0).await;
        let database_album = audio::NameDateMbz::query(&mock, id).await;
        assert_eq!(database_album, album);

        if update_album {
            let update_album = audio::NameDateMbz { mbz_id, ..Faker.fake() };
            let update_id = update_album.upsert_mock(&mock, 0).await;
            let database_update_album = audio::NameDateMbz::query(&mock, id).await;
            if mbz_id.is_some() {
                assert_eq!(id, update_id);
                assert_eq!(database_update_album, update_album);
            } else {
                // This will always insert a new row to the database
                // since there is nothing to identify an old album.
                assert_ne!(id, update_id);
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_album_upsert_no_mbz_id(#[future(awt)] mock: Mock) {
        // We want to make sure that insert the same album with no mbz_id
        // twice does not result in any error.
        let album = audio::NameDateMbz { mbz_id: None, ..Faker.fake() };
        let id = album.upsert_mock(&mock, 0).await;
        let update_id = album.upsert_mock(&mock, 0).await;
        assert_eq!(update_id, id);
    }
}
