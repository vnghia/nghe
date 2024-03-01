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
    pub music_folder_id: Uuid,
    pub path: Option<Cow<'a, str>>,
    pub file_hash: i64,
    pub file_size: i64,
}
