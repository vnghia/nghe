#![allow(clippy::option_option)]

use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;

pub use crate::schema::playlists::{self, *};

#[derive(Insertable, AsChangeset, Default)]
#[diesel(table_name = playlists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = false)]
pub struct Upsert<'a> {
    pub name: Option<Cow<'a, str>>,
    pub comment: Option<Option<Cow<'a, str>>>,
    pub public: Option<bool>,
}

mod upsert {
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{playlists, Upsert};
    use crate::database::Database;
    use crate::Error;

    impl crate::orm::upsert::Insert for Upsert<'_> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            diesel::insert_into(playlists::table)
                .values(self)
                .returning(playlists::id)
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }
    }
}
