use std::borrow::Cow;

use derive_new::new;
use uuid::Uuid;

#[derive(Debug)]
pub struct Artist<'a> {
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug, new)]
pub struct SongAlbum<'a> {
    song: Vec<Artist<'a>>,
    album: Vec<Artist<'a>>,
}

impl<'a> SongAlbum<'a> {
    pub fn song(&self) -> &Vec<Artist<'a>> {
        &self.song
    }

    pub fn album(&self) -> &Vec<Artist<'a>> {
        if self.album.is_empty() { &self.song } else { &self.album }
    }
}
