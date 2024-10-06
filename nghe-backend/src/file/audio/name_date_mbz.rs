use std::borrow::Cow;

use diesel::dsl::{exists, not};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use o2o::o2o;
use uuid::Uuid;

use super::date::Date;
use crate::database::Database;
use crate::orm::upsert::Insert as _;
use crate::orm::{albums, songs};
use crate::Error;

#[derive(Debug, o2o)]
#[try_map_owned(songs::name_date_mbz::NameDateMbz<'a>, Error)]
#[try_map_owned(albums::Data<'a>, Error)]
#[ref_try_into(songs::name_date_mbz::NameDateMbz<'a>, Error)]
#[ref_try_into(albums::Data<'a>, Error)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct NameDateMbz<'a> {
    #[ref_into(Cow::Borrowed(~.as_ref()))]
    #[cfg_attr(test, dummy(expr = "Faker.fake::<String>().into()"))]
    pub name: Cow<'a, str>,
    #[map(~.try_into()?)]
    pub date: Date,
    #[map(~.try_into()?)]
    pub release_date: Date,
    #[map(~.try_into()?)]
    pub original_release_date: Date,
    pub mbz_id: Option<Uuid>,
}

pub type Album<'a> = NameDateMbz<'a>;

impl<'a> Album<'a> {
    pub async fn upsert(&self, database: &Database, music_folder_id: Uuid) -> Result<Uuid, Error> {
        albums::Upsert { music_folder_id, data: self.try_into()? }.insert(database).await
    }

    pub async fn cleanup(database: &Database) -> Result<(), Error> {
        // Delete all albums which do not have any song associated.
        let alias_albums = diesel::alias!(albums as alias);
        diesel::delete(albums::table)
            .filter(
                albums::id.eq_any(
                    alias_albums
                        .filter(not(exists(
                            songs::table.filter(songs::album_id.eq(alias_albums.field(albums::id))),
                        )))
                        .select(alias_albums.field(albums::id)),
                ),
            )
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;

    use super::*;
    use crate::test::Mock;

    impl<'a> Album<'a> {
        pub async fn upsert_mock(&self, mock: &Mock, index: usize) -> Uuid {
            self.upsert(mock.database(), mock.music_folder(index).await.id()).await.unwrap()
        }
    }

    impl Album<'static> {
        pub async fn query(mock: &Mock, id: Uuid) -> Self {
            albums::table
                .filter(albums::id.eq(id))
                .select(albums::Data::as_select())
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
                .try_into()
                .unwrap()
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
    async fn test_album_upsert_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] mbz_id: bool,
        #[values(true, false)] update_album: bool,
    ) {
        let mbz_id = if mbz_id { Some(Faker.fake()) } else { None };
        let album = Album { mbz_id, ..Faker.fake() };
        let id = album.upsert_mock(&mock, 0).await;
        let database_album = Album::query(&mock, id).await;
        assert_eq!(database_album, album);

        if update_album {
            let update_album = Album { mbz_id, ..Faker.fake() };
            let update_id = update_album.upsert_mock(&mock, 0).await;
            let database_update_album = Album::query(&mock, id).await;
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
        let album = Album { mbz_id: None, ..Faker.fake() };
        let id = album.upsert_mock(&mock, 0).await;
        let update_id = album.upsert_mock(&mock, 0).await;
        assert_eq!(update_id, id);
    }
}
