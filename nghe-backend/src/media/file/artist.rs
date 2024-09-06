use std::borrow::Cow;

#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use uuid::Uuid;

use crate::Error;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Artist<'a> {
    #[cfg_attr(test, dummy(expr = "Faker.fake::<String>().into()"))]
    pub name: Cow<'a, str>,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Artists<'a> {
    #[cfg_attr(test, dummy(faker = "(Faker, 1..4)"))]
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
