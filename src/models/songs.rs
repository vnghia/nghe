pub use crate::schema::songs;
pub use songs::*;

use diesel::prelude::*;
use std::borrow::Cow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Selectable, Clone, PartialEq, Eq)]
#[diesel(table_name = songs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub album_id: Uuid,
    pub music_folder_id: Uuid,
    pub path: String,
    pub file_hash: i64,
    pub file_size: i64,
    pub updated_at: OffsetDateTime,
    pub scanned_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = songs)]
pub struct NewSong<'a> {
    pub title: Cow<'a, str>,
    pub album_id: Uuid,
    pub music_folder_id: Uuid,
    pub path: Cow<'a, str>,
    pub file_hash: i64,
    pub file_size: i64,
}
