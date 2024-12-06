use std::num::NonZero;

use diesel::deserialize::{self, FromSql};
use diesel::pg::PgValue;
use diesel::prelude::*;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Float;
use diesel_derives::AsChangeset;
use o2o::o2o;

use super::songs;
use crate::file::{self, audio};
use crate::{error, Error};

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[try_map_owned(audio::Property, Error)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Property {
    pub duration: audio::Duration,
    #[map(~ as _)]
    pub bitrate: i32,
    #[from(~.map(i16::from))]
    #[into(~.map(i16::try_into).transpose()?)]
    pub bit_depth: Option<i16>,
    #[map(~ as _)]
    pub sample_rate: i32,
    #[from(~.into())]
    #[into(~.try_into()?)]
    pub channel_count: i16,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[from_owned(file::Property<audio::Format>)]
#[owned_try_into(file::Property<audio::Format>, Error)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct File {
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
    pub format: audio::Format,
}

impl ToSql<Float, crate::orm::Type> for audio::Duration {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, crate::orm::Type>) -> serialize::Result {
        <f32 as ToSql<Float, crate::orm::Type>>::to_sql(&(*self).into(), &mut out.reborrow())
    }
}

impl FromSql<Float, crate::orm::Type> for audio::Duration {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        Ok(<f32 as FromSql<Float, crate::orm::Type>>::from_sql(bytes)?.into())
    }
}
