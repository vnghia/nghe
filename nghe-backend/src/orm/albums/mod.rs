use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::albums::{self, *};

pub mod date;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[cfg_attr(test, derive(Default, PartialEq, Eq))]
pub struct Foreign {
    pub music_folder_id: Uuid,
    pub cover_art_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Upsert<'a> {
    #[diesel(embed)]
    pub foreign: Foreign,
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

    impl crate::orm::upsert::Insert for Upsert<'_> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            if self.data.mbz_id.is_some() {
                diesel::insert_into(albums::table)
                    .values(self)
                    .on_conflict((albums::music_folder_id, albums::mbz_id))
                    .do_update()
                    .set((self, albums::scanned_at.eq(crate::time::now().await)))
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
                    .set((
                        albums::cover_art_id.eq(self.foreign.cover_art_id),
                        albums::scanned_at.eq(crate::time::now().await),
                    ))
                    .returning(albums::id)
                    .get_result(&mut database.get().await?)
                    .await
            }
            .map_err(Error::from)
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    impl From<Uuid> for Foreign {
        fn from(value: Uuid) -> Self {
            Self { music_folder_id: value, ..Default::default() }
        }
    }
}
