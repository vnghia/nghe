use diesel::prelude::*;
use diesel_derives::AsChangeset;

use super::songs;

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Track {
    #[diesel(column_name = track_number)]
    pub number: Option<i32>,
    #[diesel(column_name = track_total)]
    pub total: Option<i32>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct Disc {
    #[diesel(column_name = disc_number)]
    pub number: Option<i32>,
    #[diesel(column_name = disc_total)]
    pub total: Option<i32>,
}

#[derive(Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = songs, check_for_backend(crate::orm::Type))]
#[diesel(treat_none_as_null = true)]
pub struct TrackDisc {
    #[diesel(embed)]
    pub track: Track,
    #[diesel(embed)]
    pub disc: Disc,
}
