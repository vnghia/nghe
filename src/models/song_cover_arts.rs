use std::borrow::Cow;

use diesel::prelude::*;
pub use song_cover_arts::*;

pub use crate::schema::song_cover_arts;

#[derive(Insertable)]
#[diesel(table_name = song_cover_arts)]
pub struct NewSongCoverArt<'a> {
    pub format: Cow<'a, str>,
    pub file_hash: i64,
    pub file_size: i64,
}
