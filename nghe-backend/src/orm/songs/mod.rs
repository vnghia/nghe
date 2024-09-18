use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel_derives::AsChangeset;
use uuid::Uuid;

pub mod date;
pub mod name_date_mbz;
pub mod position;
mod property;

use std::borrow::Cow;
use std::str::FromStr;

use crate::file::audio;
pub use crate::schema::songs::{self, *};

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

    impl<'a> crate::orm::upsert::Trait for Upsert<'a> {
        async fn insert(self, database: &Database) -> Result<Uuid, Error> {
            diesel::insert_into(songs::table)
                .values(self)
                .returning(songs::id)
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }

        async fn update(self, database: &Database, id: Uuid) -> Result<(), Error> {
            diesel::update(songs::table)
                .filter(songs::id.eq(id))
                .set(self)
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }
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
