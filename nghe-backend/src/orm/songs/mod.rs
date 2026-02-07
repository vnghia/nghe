use std::borrow::Cow;
use std::str::FromStr;

use diesel::deserialize::{self, FromSql};
use diesel::dsl::sql;
use diesel::expression::SqlLiteral;
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types;
use diesel::sql_types::Text;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::file::audio;
pub use crate::schema::songs::{self, *};

pub mod date;
pub mod name_date_mbz;
pub mod position;
pub mod property;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[cfg_attr(test, derive(Default))]
pub struct Foreign {
    pub album_id: Uuid,
    pub cover_art_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Song<'a> {
    #[diesel(embed)]
    pub main: name_date_mbz::NameDateMbz<'a>,
    #[diesel(embed)]
    pub track_disc: position::TrackDisc,
    #[diesel(select_expression = sql("songs.languages languages"))]
    #[diesel(select_expression_type = SqlLiteral<sql_types::Array<sql_types::Text>>)]
    pub languages: Vec<Cow<'a, str>>,
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

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Upsert<'a> {
    #[diesel(embed)]
    pub foreign: Foreign,
    pub relative_path: Cow<'a, str>,
    #[diesel(embed)]
    pub data: Data<'a>,
}

#[derive(Debug, Clone, Copy, Queryable, Selectable)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
pub struct Time {
    pub scanned_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, Queryable, Selectable)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
pub struct IdTime {
    pub id: Uuid,
    #[diesel(embed)]
    pub time: Time,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
pub struct IdPath {
    pub id: Uuid,
    #[diesel(embed)]
    pub time: Time,
    pub relative_path: String,
}

mod upsert {
    use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
    use diesel_async::RunQueryDsl;
    use uuid::Uuid;

    use super::{Upsert, songs};
    use crate::Error;
    use crate::database::Database;

    impl crate::orm::upsert::Insert for Upsert<'_> {
        async fn insert(&self, database: &Database) -> Result<Uuid, Error> {
            // Set `scanned_at` so it can still return something when there is a conflict.
            let song_id = diesel::insert_into(songs::table)
                .values(self)
                .on_conflict_do_nothing()
                .returning(songs::id)
                .get_result(&mut database.get().await?)
                .await
                .optional()?;
            Ok(if let Some(song_id) = song_id {
                song_id
            } else {
                // If there is a conflict, it means that we are doing multiple scans at the same
                // time. We will just return the `song_id` inserted by another process.
                songs::table
                    .filter(songs::album_id.eq(self.foreign.album_id))
                    .filter(songs::relative_path.eq(self.relative_path.as_str()))
                    .select(songs::id)
                    .get_result(&mut database.get().await?)
                    .await?
            })
        }
    }

    impl crate::orm::upsert::Update for Upsert<'_> {
        async fn update(&self, database: &Database, id: Uuid) -> Result<(), Error> {
            diesel::update(songs::table)
                .filter(songs::id.eq(id))
                .set(self)
                .execute(&mut database.get().await?)
                .await?;
            Ok(())
        }
    }

    impl crate::orm::upsert::Upsert for Upsert<'_> {}
}

impl ToSql<Text, super::Type> for audio::Format {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, super::Type>) -> serialize::Result {
        <str as ToSql<Text, super::Type>>::to_sql(self.into(), out)
    }
}

impl FromSql<Text, super::Type> for audio::Format {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        Ok(audio::Format::from_str(core::str::from_utf8(bytes.as_bytes())?)?)
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    impl From<Uuid> for Foreign {
        fn from(value: Uuid) -> Self {
            Self { album_id: value, ..Default::default() }
        }
    }
}
