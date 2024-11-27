use diesel::prelude::*;
use diesel_derives::AsChangeset;
use nghe_api::id3;
use o2o::o2o;

use super::albums;
use crate::Error;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[owned_try_into(id3::date::Date, Error)]
pub struct Date {
    #[try_into(~.map(i16::try_into).transpose()?)]
    pub year: Option<i16>,
    #[try_into(~.map(i16::try_into).transpose()?)]
    pub month: Option<i16>,
    #[try_into(~.map(i16::try_into).transpose()?)]
    pub day: Option<i16>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[owned_try_into(id3::date::Date, Error)]
pub struct Release {
    #[try_into(~.map(i16::try_into).transpose()?)]
    #[diesel(column_name = release_year)]
    pub year: Option<i16>,
    #[try_into(~.map(i16::try_into).transpose()?)]
    #[diesel(column_name = release_month)]
    pub month: Option<i16>,
    #[try_into(~.map(i16::try_into).transpose()?)]
    #[diesel(column_name = release_day)]
    pub day: Option<i16>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset, o2o)]
#[diesel(table_name = albums, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
#[owned_try_into(id3::date::Date, Error)]
pub struct OriginalRelease {
    #[try_into(~.map(i16::try_into).transpose()?)]
    #[diesel(column_name = original_release_year)]
    pub year: Option<i16>,
    #[try_into(~.map(i16::try_into).transpose()?)]
    #[diesel(column_name = original_release_month)]
    pub month: Option<i16>,
    #[try_into(~.map(i16::try_into).transpose()?)]
    #[diesel(column_name = original_release_day)]
    pub day: Option<i16>,
}
