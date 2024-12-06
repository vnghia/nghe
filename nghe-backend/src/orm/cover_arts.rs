use std::borrow::Cow;
use std::num::NonZero;
use std::str::FromStr;

use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel_derives::AsChangeset;
use o2o::o2o;

use crate::file::{self, picture};
pub use crate::schema::cover_arts::{self, *};
use crate::{error, Error};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[from_owned(file::Property<picture::Format>)]
#[owned_try_into(file::Property<picture::Format>, Error)]
#[diesel(table_name = cover_arts, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Property {
    #[from(~.cast_signed())]
    #[into(~.cast_unsigned())]
    #[diesel(column_name = file_hash)]
    pub hash: i64,
    #[from(~.get().cast_signed())]
    #[into(NonZero::new(~.cast_unsigned()).ok_or_else(
        || error::Kind::DatabaseCorruptionDetected
    )?)]
    #[diesel(column_name = file_size)]
    pub size: i32,
    pub format: picture::Format,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = cover_arts, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'s> {
    pub source: Option<Cow<'s, str>>,
    #[diesel(embed)]
    pub property: Property,
}

mod upsert {
    use diesel::ExpressionMethods;
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{cover_arts, Upsert};
    use crate::database::Database;
    use crate::Error;

    impl crate::orm::upsert::Insert for Upsert<'_> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            diesel::insert_into(cover_arts::table)
                .values(self)
                .on_conflict((cover_arts::source, cover_arts::file_hash, cover_arts::file_size))
                .do_update()
                .set(cover_arts::format.eq(self.property.format))
                .returning(cover_arts::id)
                .get_result(&mut database.get().await?)
                .await
                .map_err(Error::from)
        }
    }
}

impl ToSql<Text, super::Type> for picture::Format {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, super::Type>) -> serialize::Result {
        <str as ToSql<Text, super::Type>>::to_sql(self.into(), out)
    }
}

impl FromSql<Text, super::Type> for picture::Format {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        Ok(picture::Format::from_str(core::str::from_utf8(bytes.as_bytes())?)?)
    }
}
