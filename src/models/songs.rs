pub use crate::schema::songs;
pub use songs::*;

use diesel::prelude::*;
use std::borrow::Cow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Selectable, Clone, PartialEq)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub duration: f32,
    pub album_id: Uuid,
    pub music_folder_id: Uuid,
    pub path: String,
    pub file_hash: i64,
    pub file_size: i64,
    pub updated_at: OffsetDateTime,
    pub scanned_at: OffsetDateTime,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = songs)]
pub struct NewOrUpdateSong<'a> {
    pub title: Cow<'a, str>,
    pub duration: f32,
    pub album_id: Uuid,
    pub track_number: Option<i32>,
    pub track_total: Option<i32>,
    pub disc_number: Option<i32>,
    pub disc_total: Option<i32>,
    pub year: Option<i16>,
    pub month: Option<i16>,
    pub day: Option<i16>,
    pub release_year: Option<i16>,
    pub release_month: Option<i16>,
    pub release_day: Option<i16>,
    pub original_release_year: Option<i16>,
    pub original_release_month: Option<i16>,
    pub original_release_day: Option<i16>,
    pub music_folder_id: Uuid,
    pub path: Option<Cow<'a, str>>,
    pub file_hash: i64,
    pub file_size: i64,
}
