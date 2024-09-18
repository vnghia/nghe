use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub mod date;

use std::borrow::Cow;

pub use crate::schema::albums::{self, *};

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
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{albums, Upsert};
    use crate::database::Database;
    use crate::Error;

    impl<'a> crate::orm::upsert::Trait for Upsert<'a> {
        async fn insert(self, database: &Database) -> Result<Uuid, Error> {
            diesel::insert_into(albums::table)
                .values(self)
                .returning(albums::id)
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }

        async fn update(self, database: &Database, id: Uuid) -> Result<(), Error> {
            diesel::update(albums::table)
                .filter(albums::id.eq(id))
                .set(self)
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
