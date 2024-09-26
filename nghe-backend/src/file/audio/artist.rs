use std::borrow::Cow;

#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use futures_lite::{stream, StreamExt};
use o2o::o2o;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::artists;
use crate::orm::upsert::Insert as _;
use crate::Error;

#[derive(Debug, o2o)]
#[from_owned(artists::Data<'a>)]
#[ref_into(artists::Data<'a>)]
#[cfg_attr(test, derive(PartialEq, Eq, Dummy, Clone))]
pub struct Artist<'a> {
    #[ref_into(Cow::Borrowed(~.as_ref()))]
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
    pub compilation: bool,
}

impl<'a> Artist<'a> {
    pub fn index(&self, prefixes: &[impl AsRef<str>]) -> Result<char, Error> {
        let mut iter = prefixes.iter();
        let name = loop {
            match iter.next() {
                Some(prefix) => {
                    if let Some(name) = self.name.strip_prefix(prefix.as_ref()) {
                        break name;
                    }
                }
                None => break self.name.as_ref(),
            }
        };
        name.nfkd().next().ok_or_else(|| Error::MediaArtistNameEmpty).map(|c| {
            if c.is_ascii_alphabetic() {
                c.to_ascii_uppercase()
            } else if c.is_numeric() {
                '#'
            } else if !c.is_alphabetic() {
                '*'
            } else {
                c
            }
        })
    }

    pub async fn upsert(
        &self,
        database: &Database,
        prefixes: &[impl AsRef<str>],
    ) -> Result<Uuid, Error> {
        artists::Upsert { index: self.index(prefixes)?.to_string().into(), data: self.into() }
            .insert(database)
            .await
    }

    async fn upserts(
        database: &Database,
        artists: &[Self],
        prefixes: &[impl AsRef<str>],
    ) -> Result<Vec<Uuid>, Error> {
        stream::iter(artists)
            .then(async |artist| artist.upsert(database, prefixes).await)
            .try_collect()
            .await
    }
}

impl<'a> Artists<'a> {
    pub fn new(
        song: Vec<Artist<'a>>,
        album: Vec<Artist<'a>>,
        compilation: bool,
    ) -> Result<Self, Error> {
        if song.is_empty() {
            Err(Error::MediaSongArtistEmpty)
        } else {
            Ok(Self { song, album, compilation })
        }
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("The One", &["The ", "A "], 'O')]
    #[case("The 1", &["The ", "A "], '#')]
    #[case("The one", &["The ", "A "], 'O')]
    #[case("狼", &["The ", "A "], '狼')]
    #[case("é", &["The ", "A "], 'E')]
    #[case("ド", &["The ", "A "], 'ト')]
    #[case("ａ", &["The ", "A "], 'A')]
    #[case("%", &["The ", "A "], '*')]
    fn test_index(#[case] name: &str, #[case] prefixes: &[&str], #[case] index: char) {
        assert_eq!(Artist::from(name).index(prefixes).unwrap(), index);
    }
}
