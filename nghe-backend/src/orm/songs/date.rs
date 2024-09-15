use diesel::prelude::*;

use super::songs;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Date {
    pub year: Option<i16>,
    pub month: Option<i16>,
    pub day: Option<i16>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Release {
    #[diesel(column_name = release_year)]
    pub year: Option<i16>,
    #[diesel(column_name = release_month)]
    pub month: Option<i16>,
    #[diesel(column_name = release_day)]
    pub day: Option<i16>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct OriginalRelease {
    #[diesel(column_name = original_release_year)]
    pub year: Option<i16>,
    #[diesel(column_name = original_release_month)]
    pub month: Option<i16>,
    #[diesel(column_name = original_release_day)]
    pub day: Option<i16>,
}
