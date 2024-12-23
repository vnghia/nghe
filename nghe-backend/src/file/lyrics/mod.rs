use core::str;
use std::borrow::Cow;

use alrc::AdvancedLrc;
use isolang::Language;
use lofty::id3::v2::{BinaryFrame, SynchronizedTextFrame, UnsynchronizedTextFrame};

use super::audio;
use crate::{Error, error};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Clone))]
pub enum Lines<'a> {
    Unsync(Vec<Cow<'a, str>>),
    Sync(Vec<(audio::Duration, Cow<'a, str>)>),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Clone))]
pub struct Lyrics<'a> {
    pub description: Option<Cow<'a, str>>,
    pub language: Language,
    pub lines: Lines<'a>,
}

impl<'a> FromIterator<&'a str> for Lines<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::Unsync(iter.into_iter().map(Cow::Borrowed).collect())
    }
}

impl FromIterator<(time::Duration, String)> for Lines<'_> {
    fn from_iter<T: IntoIterator<Item = (time::Duration, String)>>(iter: T) -> Self {
        Self::Sync(
            iter.into_iter().map(|(duration, text)| (duration.into(), text.into())).collect(),
        )
    }
}

impl FromIterator<(u32, String)> for Lines<'_> {
    fn from_iter<T: IntoIterator<Item = (u32, String)>>(iter: T) -> Self {
        iter.into_iter()
            .map(|(duration, text)| (time::Duration::milliseconds(duration.into()), text))
            .collect()
    }
}

impl<'a> TryFrom<&'a UnsynchronizedTextFrame<'_>> for Lyrics<'a> {
    type Error = Error;

    fn try_from(frame: &'a UnsynchronizedTextFrame<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            description: Some(frame.description.as_str().into()),
            language: str::from_utf8(&frame.language)?.parse().map_err(error::Kind::from)?,
            lines: frame.content.lines().collect(),
        })
    }
}

impl<'a> TryFrom<&'a BinaryFrame<'_>> for Lyrics<'a> {
    type Error = Error;

    fn try_from(frame: &'a BinaryFrame<'_>) -> Result<Self, Error> {
        let frame = SynchronizedTextFrame::parse(&frame.data, frame.flags())?;
        Ok(Self {
            description: frame.description.map(Cow::Owned),
            language: str::from_utf8(&frame.language)?.parse().map_err(error::Kind::from)?,
            lines: frame.content.into_iter().collect(),
        })
    }
}

impl<'a> Lyrics<'a> {
    pub fn from_unsync_text(content: &'a str) -> Self {
        Self { description: None, language: Language::Und, lines: content.lines().collect() }
    }

    pub fn from_sync_text(content: &str) -> Result<Self, Error> {
        let lrc = AdvancedLrc::parse(content).map_err(error::Kind::InvalidLyricsLrcFormat)?;
        Ok(Self {
            description: lrc.metadata.get("desc").map(String::from).map(Cow::Owned),
            language: lrc
                .metadata
                .get("lang")
                .map_or(Ok(Language::Und), |language| language.parse())
                .map_err(error::Kind::from)?,
            lines: lrc
                .lines
                .into_iter()
                .map(|line| {
                    let duration: time::Duration = line.time.to_duration().try_into()?;
                    Ok::<_, Error>((duration, line.text))
                })
                .try_collect()?,
        })
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use fake::{Dummy, Fake, Faker};
    use itertools::Itertools;

    use super::*;

    impl FromIterator<String> for Lines<'_> {
        fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
            Self::Unsync(iter.into_iter().map(Cow::Owned).collect())
        }
    }

    impl Dummy<Faker> for Lyrics<'_> {
        fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
            if config.fake_with_rng(rng) {
                Self {
                    description: config.fake_with_rng::<Option<String>, _>(rng).map(Cow::Owned),
                    language: Language::from_usize((0..=7915).fake()).unwrap(),
                    lines: fake::vec![(u32, String); 1..=5].into_iter().sorted().collect(),
                }
            } else {
                Self {
                    description: None,
                    language: Language::Und,
                    lines: fake::vec![String; 1..=5].into_iter().collect(),
                }
            }
        }
    }
}
