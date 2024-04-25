use std::borrow::Cow;
use std::path::{Path, PathBuf};

use concat_string::concat_string;
pub use cover_arts::*;
use diesel::prelude::*;

pub use crate::schema::cover_arts;
use crate::utils::fs::path::hash_size_to_path;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = cover_arts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewCoverArt<'a> {
    pub format: Cow<'a, str>,
    pub file_hash: i64,
    pub file_size: i32,
}

impl<'a> NewCoverArt<'a> {
    pub fn to_path<P: AsRef<Path>>(&'a self, song_art_dir: P) -> PathBuf {
        hash_size_to_path(song_art_dir, self.file_hash as _, self.file_size as _)
            .join(concat_string!("cover.", self.format))
    }
}

pub type CoverArt<'a> = NewCoverArt<'a>;
