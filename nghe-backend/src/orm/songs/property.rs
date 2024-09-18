use diesel::prelude::*;
use diesel_derives::AsChangeset;
use o2o::o2o;

use super::songs;
use crate::file::{audio, property};
use crate::Error;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[try_map_owned(audio::Property, Error)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Property {
    pub duration: f32,
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
#[map_owned(property::File<audio::Format>)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct File {
    #[map(~ as _)]
    #[diesel(column_name = file_hash)]
    hash: i64,
    #[map(~ as _)]
    #[diesel(column_name = file_size)]
    size: i32,
    format: audio::Format,
}
