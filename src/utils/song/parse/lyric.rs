use std::borrow::Cow;
use std::io::BufReader;
use std::ops::Deref;
use std::str::FromStr;

use anyhow::Result;
#[cfg(test)]
use fake::{Dummy, Fake, Faker};
use isolang::Language;
#[cfg(test)]
use itertools::Itertools;
use lofty::id3::v2::{Id3v2Version, SynchronizedText, UnsynchronizedTextFrame};
use lrc::Lyrics;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use crate::models::*;
use crate::OSError;

#[derive(Debug)]
#[cfg_attr(test, derive(Clone, Dummy, PartialEq, Eq))]
pub enum LyricLines {
    Unsynced(#[cfg_attr(test, dummy(faker = "(Faker, 1..=5)"))] Vec<String>),
    Synced(
        #[cfg_attr(
            test,
            dummy(expr = "fake::vec![(u16, String); 1..=5].into_iter().map(|(s, v)| (s as u32 * \
                          10, v)).sorted().collect()")
        )]
        Vec<(u32, String)>,
    ),
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone, Dummy))]
pub struct SongLyric {
    pub description: String,
    #[cfg_attr(test, dummy(expr = "Language::from_usize((0..=7915).fake()).unwrap()"))]
    pub language: Language,
    pub lines: LyricLines,
    pub external: bool,
    #[cfg_attr(test, dummy(expr = "0"))]
    pub lyric_hash: u64,
    #[cfg_attr(test, dummy(expr = "0"))]
    pub lyric_size: u64,
}

impl SongLyric {
    pub fn from_synced_bytes(data: &[u8]) -> Result<Self> {
        let lyric_hash = xxh3_64(data);
        let lyric_size = data.len() as _;

        let parsed = SynchronizedText::parse(data)?;
        let lines = parsed.content.into();
        Ok(Self {
            description: parsed.description.unwrap_or_default(),
            language: Language::from_str(
                &parsed.language.into_iter().map(|u| u as char).collect::<String>(),
            )?,
            lines,
            external: false,
            lyric_hash,
            lyric_size,
        })
    }

    pub fn from_unsynced_bytes(data: &[u8], version: Id3v2Version) -> Result<Self> {
        let lyric_hash = xxh3_64(data);
        let lyric_size = data.len() as _;

        let parsed = UnsynchronizedTextFrame::parse(&mut BufReader::new(data), version)?
            .ok_or_else(|| OSError::NotFound("USLT".into()))?;
        let lines = parsed.content.lines().collect();
        Ok(Self {
            description: parsed.description,
            language: Language::from_str(
                &parsed.language.into_iter().map(|u| u as char).collect::<String>(),
            )?,
            lines,
            external: false,
            lyric_hash,
            lyric_size,
        })
    }

    fn extract_synced_metadata<'a>(lyric: &'a Lyrics, key: &str) -> Option<&'a str> {
        lyric
            .metadata
            .iter()
            .filter_map(|t| if t.label().deref().deref() == key { Some(t.text()) } else { None })
            .next()
    }

    pub fn from_str(data: &str, external: bool) -> Result<Self> {
        let lyric_hash = xxh3_64(data.as_bytes());
        let lyric_size = data.len() as _;

        if let Ok(parsed) = Lyrics::from_str(data)
            && !parsed.get_timed_lines().is_empty()
        {
            let description = Self::extract_synced_metadata(&parsed, "desc")
                .map_or(String::default(), String::from);
            let language = Self::extract_synced_metadata(&parsed, "lang")
                .map_or(Ok(Language::Und), Language::from_str)?;

            let lines = parsed
                .get_timed_lines()
                .iter()
                .map(|(t, l)| (t.get_timestamp() as u32, l))
                .collect();

            Ok(Self { description, language, lines, external, lyric_hash, lyric_size })
        } else {
            Ok(Self {
                description: String::default(),
                language: Language::Und,
                lines: data.lines().collect(),
                external,
                lyric_hash,
                lyric_size,
            })
        }
    }
}

impl<T, S, V> From<T> for LyricLines
where
    S: Into<u32>,
    V: ToString,
    T: IntoIterator<Item = (S, V)>,
{
    fn from(values: T) -> Self {
        Self::Synced(values.into_iter().map(|(s, v)| (s.into(), v.to_string())).collect())
    }
}

impl<S, V> FromIterator<(S, V)> for LyricLines
where
    S: Into<u32>,
    V: ToString,
{
    fn from_iter<T: IntoIterator<Item = (S, V)>>(iter: T) -> Self {
        Self::Synced(iter.into_iter().map(|(s, v)| (s.into(), v.to_string())).collect())
    }
}

impl FromIterator<String> for LyricLines {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::Unsynced(iter.into_iter().filter(|s| !s.is_empty()).collect())
    }
}

impl<'a> FromIterator<&'a str> for LyricLines {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        iter.into_iter().map(String::from).collect()
    }
}

impl LyricLines {
    fn unzip(&self) -> (Option<Vec<Option<i32>>>, Vec<Option<Cow<'_, str>>>) {
        match self {
            LyricLines::Unsynced(ref lines) => {
                (None, lines.iter().map(|v| Some(v.as_str().into())).collect())
            }
            LyricLines::Synced(ref lines) => {
                let (line_starts, line_values): (Vec<_>, Vec<_>) =
                    lines.iter().map(|(s, v)| (Some(*s as _), Some(v.as_str().into()))).unzip();
                (Some(line_starts), line_values)
            }
        }
    }
}

impl SongLyric {
    pub fn as_key(&self, song_id: Uuid) -> lyrics::LyricKey<'_> {
        lyrics::LyricKey {
            song_id,
            description: self.description.as_str().into(),
            language: self.language.to_639_3().into(),
            external: self.external,
        }
    }

    pub fn as_update(&self) -> lyrics::UpdateLyric<'_> {
        let (line_starts, line_values) = self.lines.unzip();
        lyrics::UpdateLyric {
            line_starts,
            line_values,
            lyric_hash: self.lyric_hash as _,
            lyric_size: self.lyric_size as _,
        }
    }
}

#[cfg(test)]
mod test {
    use lrc::{IDTag, TimeTag};
    use nghe_types::open_subsonic::common::id3::response::LyricId3;

    use super::*;

    impl std::fmt::Display for SongLyric {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let result = match &self.lines {
                LyricLines::Unsynced(lines) => lines.join("\n"),
                LyricLines::Synced(lines) => {
                    let mut lyric = Lyrics::new();
                    lines.iter().for_each(|(s, v)| {
                        lyric.add_timed_line(TimeTag::new(*s), v).unwrap();
                    });
                    lyric
                        .metadata
                        .insert(IDTag::from_string("lang", self.language.to_639_3()).unwrap());
                    lyric.metadata.insert(IDTag::from_string("desc", &self.description).unwrap());
                    lyric.to_string()
                }
            };
            write!(f, "{}", result)
        }
    }

    impl From<(LyricId3, String, bool)> for SongLyric {
        fn from((lyric, description, external): (LyricId3, String, bool)) -> Self {
            let lines = if lyric.synced {
                lyric.line.into_iter().map(|l| (l.start.unwrap(), l.value)).collect()
            } else {
                lyric.line.into_iter().map(|l| l.value).collect()
            };
            Self {
                description,
                language: lyric.lang,
                lines,
                external,
                lyric_hash: 0,
                lyric_size: 0,
            }
        }
    }

    impl PartialEq for SongLyric {
        fn eq(&self, other: &Self) -> bool {
            match self.lines {
                LyricLines::Unsynced(_) => self.lines == other.lines,
                LyricLines::Synced(_) => {
                    self.description == other.description
                        && self.language == other.language
                        && self.lines == other.lines
                        && self.external == other.external
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::asset::get_asset_dir;

    #[test]
    fn test_from_str() {
        let sample_dir = get_asset_dir().join("test").join("lyric");

        let lyric = SongLyric::from_str(
            &std::fs::read_to_string(sample_dir.join("synced.lrc")).unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(lyric.description, "Lyric");
        assert_eq!(lyric.language, Language::Eng);
        assert_eq!(
            lyric.lines,
            vec![
                (1020_u32, "Hello hi"),
                (3040_u32, "Bonjour salut"),
                (5060_u32, "おはよう こんにちは")
            ]
            .into_iter()
            .collect()
        );

        let lyric = SongLyric::from_str(
            &std::fs::read_to_string(sample_dir.join("unsynced.lrc")).unwrap(),
            true,
        )
        .unwrap();
        assert_eq!(
            lyric.lines,
            vec!["Hello hi", "Bonjour salut", "おはよう こんにちは"].into_iter().collect()
        );
    }
}
