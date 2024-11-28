#![allow(clippy::option_option)]

use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;
use o2o::o2o;

pub use crate::schema::playlists::{self, *};

#[derive(Insertable, AsChangeset, Default, o2o)]
#[diesel(table_name = playlists, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = false)]
#[from_ref(nghe_api::playlists::update_playlist::Request)]
pub struct Upsert<'a> {
    #[from(~.as_ref().map(|value| value.as_str().into()))]
    pub name: Option<Cow<'a, str>>,
    #[from(~.as_ref().map(
        |value| if value.is_empty() { None } else { Some(value.as_str().into()) }
    ))]
    pub comment: Option<Option<Cow<'a, str>>>,
    pub public: Option<bool>,
}

mod upsert {
    use diesel::ExpressionMethods;
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

    impl crate::orm::upsert::Update for Upsert<'_> {
        async fn update(&self, database: &Database, id: Uuid) -> Result<(), Error> {
            diesel::update(playlists::table)
                .filter(playlists::id.eq(id))
                .set(self)
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
