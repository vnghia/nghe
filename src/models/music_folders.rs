use std::borrow::Cow;

use diesel::prelude::*;
pub use music_folders::*;
use nghe_proc_macros::add_convert_types;
use uuid::Uuid;

pub use crate::schema::music_folders;

#[add_convert_types(into = nghe_types::music_folder::MusicFolder, skips(path))]
#[add_convert_types(into = nghe_types::music_folder::MusicFolderPath)]
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
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMusicFolder<'a> {
    pub path: Cow<'a, str>,
    pub name: Cow<'a, str>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = music_folders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpsertMusicFolder<'a> {
    pub name: Option<Cow<'a, str>>,
    pub path: Option<Cow<'a, str>>,
}
