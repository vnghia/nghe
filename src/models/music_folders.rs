pub use crate::schema::music_folders;
pub use music_folders::*;

use diesel::prelude::*;
use serde::Serialize;
use std::borrow::Cow;
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Serialize)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, PartialEq, Eq, PartialOrd, Ord))]
pub struct MusicFolder {
    pub id: Uuid,
    #[serde(rename = "name")]
    pub path: String,
}

#[derive(Insertable)]
#[diesel(table_name = music_folders)]
pub struct NewMusicFolder<'a> {
    pub path: Cow<'a, str>,
}
