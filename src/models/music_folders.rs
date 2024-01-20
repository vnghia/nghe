pub use crate::schema::music_folders;
pub use music_folders::*;

use diesel::prelude::*;
use serde::Serialize;
use std::borrow::Cow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(
    Debug, Identifiable, Queryable, Selectable, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MusicFolder {
    pub id: Uuid,
    #[serde(rename = "name")]
    pub path: String,
    #[serde(skip_serializing)]
    pub updated_at: OffsetDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = music_folders)]
pub struct NewMusicFolder<'a> {
    pub path: Cow<'a, str>,
}
