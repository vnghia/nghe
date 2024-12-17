use std::borrow::Cow;

use diesel::prelude::*;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub use crate::schema::artist_informations::{self, *};

#[derive(Debug, Default, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = artist_informations, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Spotify<'a> {
    #[diesel(column_name = spotify_id)]
    pub id: Option<Cow<'a, str>>,
    pub cover_art_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = artist_informations, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'s> {
    #[diesel(embed)]
    pub spotify: Spotify<'s>,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Data, artist_informations};
    use crate::Error;
    use crate::database::Database;

    impl crate::orm::upsert::Update for Data<'_> {
        async fn update(&self, database: &Database, id: Uuid) -> Result<(), Error> {
            diesel::insert_into(artist_informations::table)
                .values((artist_informations::artist_id.eq(id), self))
                .on_conflict(artist_informations::artist_id)
                .do_update()
                .set(self)
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
}
