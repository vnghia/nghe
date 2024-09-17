use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;
use diesel_derives::AsChangeset;

pub mod date;
pub mod name_date_mbz;
pub mod position;

use std::borrow::Cow;
use std::str::FromStr;

use crate::file::audio;
pub use crate::schema::songs::{self, *};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Data<'a> {
    #[diesel(embed)]
    pub main: name_date_mbz::NameDateMbz<'a>,
    #[diesel(embed)]
    pub track_disc: position::TrackDisc,
    pub languages: Vec<Option<Cow<'a, str>>>,
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
