use std::borrow::Cow;

use uuid::Uuid;

use crate::Error;

#[derive(Debug)]
pub struct Artist<'a> {
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug)]
pub struct Artists<'a> {
    song: Vec<Artist<'a>>,
    album: Vec<Artist<'a>>,
}

impl<'a> Artists<'a> {
    pub fn new(song: Vec<Artist<'a>>, album: Vec<Artist<'a>>) -> Result<Self, Error> {
        if song.is_empty() { Err(Error::MediaSongArtistEmpty) } else { Ok(Self { song, album }) }
    }

    pub fn song(&self) -> &Vec<Artist<'a>> {
        &self.song
    }

    pub fn album(&self) -> &Vec<Artist<'a>> {
        if self.album.is_empty() { &self.song } else { &self.album }
    }
}
