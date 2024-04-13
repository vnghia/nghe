use std::borrow::Cow;

use diesel::prelude::*;
pub use music_folders::*;
use uuid::Uuid;

pub use crate::schema::music_folders;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(Clone, PartialEq, Eq, PartialOrd, Ord))]
pub struct MusicFolder {
    pub id: Uuid,
    pub name: String,
    pub path: String,
}

#[derive(Insertable)]
#[diesel(table_name = music_folders)]
pub struct NewMusicFolder<'a> {
    pub path: Cow<'a, str>,
    pub name: Cow<'a, str>,
}

impl From<MusicFolder> for nghe_types::browsing::MusicFolder {
    fn from(value: MusicFolder) -> Self {
        Self { id: value.id, name: value.name }
    }
}

impl From<MusicFolder> for nghe_types::browsing::MusicFolderPath {
    fn from(value: MusicFolder) -> Self {
        Self { id: value.id, name: value.name, path: value.path }
    }
}
