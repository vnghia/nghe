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

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = artists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Artist<'a> {
    pub id: Uuid,
    #[diesel(embed)]
    pub data: Data<'a>,
}

mod upsert {
    use diesel::{DecoratableTarget, ExpressionMethods};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Upsert, artists};
    use crate::Error;
    use crate::database::Database;

    impl crate::orm::upsert::Insert for Upsert<'_> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            if self.data.mbz_id.is_some() {
                diesel::insert_into(artists::table)
                    .values(self)
                    .on_conflict(artists::mbz_id)
                    .do_update()
                    .set((self, artists::scanned_at.eq(crate::time::now().await)))
                    .returning(artists::id)
                    .get_result(&mut database.get().await?)
                    .await
            } else {
                diesel::insert_into(artists::table)
                    .values(self)
                    .on_conflict(artists::name)
                    .filter_target(artists::mbz_id.is_null())
                    .do_update()
                    .set(artists::scanned_at.eq(crate::time::now().await))
                    .returning(artists::id)
                    .get_result(&mut database.get().await?)
                    .await
            }
            .map_err(Error::from)
        }
    }
}
