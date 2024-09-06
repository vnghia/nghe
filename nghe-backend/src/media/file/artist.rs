use std::borrow::Cow;

use uuid::Uuid;

use crate::Error;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Artist<'a> {
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Artists<'a> {
    pub song: Vec<Artist<'a>>,
    pub album: Vec<Artist<'a>>,
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

#[cfg(test)]
mod test {
    use super::*;

    impl<'a> From<&'a str> for Artist<'a> {
        fn from(value: &'a str) -> Self {
            Self { name: value.into(), mbz_id: None }
        }
    }

    impl<'a> From<(&'a str, Uuid)> for Artist<'a> {
        fn from(value: (&'a str, Uuid)) -> Self {
            Self { name: value.0.into(), mbz_id: Some(value.1) }
        }
    }
}
