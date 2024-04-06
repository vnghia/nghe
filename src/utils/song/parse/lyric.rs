use std::io::BufReader;
use std::ops::Deref;
use std::str::FromStr;

use anyhow::Result;
use isolang::Language;
use lofty::id3::v2::{Id3v2Version, SynchronizedText, UnsynchronizedTextFrame};
use lrc::Lyrics;
use uuid::Uuid;
use xxhash_rust::xxh3::xxh3_64;

use crate::models::*;
use crate::OSError;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct LyricLine {
    pub start: Option<u32>,
    pub value: String,
}

#[derive(Debug)]
pub struct SongLyric {
    pub description: String,
    pub language: Language,
    pub lines: Vec<LyricLine>,
    pub lyric_source: String,
    pub lyric_hash: u64,
    pub lyric_size: u64,
}

impl SongLyric {
    pub fn from_synced_bytes(data: &[u8]) -> Result<Self> {
        let lyric_hash = xxh3_64(data);
        let lyric_size = data.len() as _;

        let parsed = SynchronizedText::parse(data)?;
        let lines = parsed.content.into_iter().map(LyricLine::from).collect();
        Ok(Self {
            description: parsed.description.unwrap_or_default(),
            language: Language::from_str(
                &parsed.language.into_iter().map(|u| u as char).collect::<String>(),
            )?,
            lines,
            lyric_source: "SYLT".into(),
            lyric_hash,
            lyric_size,
        })
    }

    pub fn from_unsynced_bytes(data: &[u8], version: Id3v2Version) -> Result<Self> {
        let lyric_hash = xxh3_64(data);
        let lyric_size = data.len() as _;

        let parsed = UnsynchronizedTextFrame::parse(&mut BufReader::new(data), version)?
            .ok_or_else(|| OSError::NotFound("USLT".into()))?;
        let lines = parsed.content.lines().map(LyricLine::from).collect();
        Ok(Self {
            description: parsed.description,
            language: Language::from_str(
                &parsed.language.into_iter().map(|u| u as char).collect::<String>(),
            )?,
            lines,
            lyric_source: "USLT".into(),
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

    pub fn from_str(data: &str, lyric_source: String, force_unsynced: bool) -> Result<Self> {
        let lyric_hash = xxh3_64(data.as_bytes());
        let lyric_size = data.len() as _;

        if !force_unsynced && let Ok(parsed) = Lyrics::from_str(data) {
            let description = Self::extract_synced_metadata(&parsed, "des")
                .map_or(String::default(), String::from);
            let language = Self::extract_synced_metadata(&parsed, "lang")
                .map_or(Ok(Language::Und), Language::from_str)?;

            let lines = parsed
                .get_timed_lines()
                .iter()
                .map(|(t, l)| LyricLine {
                    start: Some(t.get_timestamp() as _),
                    value: l.to_string(),
                })
                .collect();

            Ok(Self { description, language, lines, lyric_source, lyric_hash, lyric_size })
        } else {
            Ok(Self {
                description: String::default(),
                language: Language::Und,
                lines: data
                    .lines()
                    .filter_map(|l| if l.is_empty() { None } else { Some(l.into()) })
                    .collect(),
                lyric_source,
                lyric_hash,
                lyric_size,
            })
        }
    }
}

impl<S, V> From<(S, V)> for LyricLine
where
    S: Into<Option<u32>>,
    V: ToString,
{
    fn from(value: (S, V)) -> Self {
        Self { start: value.0.into(), value: value.1.to_string() }
    }
}

impl From<&str> for LyricLine {
    fn from(value: &str) -> Self {
        Self { start: None, value: value.to_owned() }
    }
}

impl SongLyric {
    pub fn as_key(&self, song_id: Uuid) -> lyrics::LyricKey<'_> {
        lyrics::LyricKey {
            song_id,
            description: self.description.as_str().into(),
            language: self.language.to_639_3().into(),
            lyric_source: self.lyric_source.as_str().into(),
        }
    }

    pub fn as_update(&self) -> lyrics::UpdateLyric<'_> {
        let (line_starts, line_values): (Vec<_>, Vec<_>) = self
            .lines
            .iter()
            .map(|l| (l.start.map(|i| Some(i as _)), Some(l.value.as_str())))
            .unzip();
        lyrics::UpdateLyric {
            line_starts: line_starts.into_iter().collect::<Option<Vec<_>>>().map(|v| v.into()),
            line_values: line_values.into(),
            lyric_hash: self.lyric_hash as _,
            lyric_size: self.lyric_size as _,
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
            "lrc".into(),
            false,
        )
        .unwrap();
        assert_eq!(lyric.description, "Lyric");
        assert_eq!(lyric.language, Language::Eng);
        assert_eq!(
            lyric.lines,
            vec![
                (1020, "Hello hi").into(),
                (3040, "Bonjour salut").into(),
                (5060, "おはよう こんにちは").into()
            ]
        );

        let lyric = SongLyric::from_str(
            &std::fs::read_to_string(sample_dir.join("unsynced.lrc")).unwrap(),
            "lrc".into(),
            true,
        )
        .unwrap();
        assert_eq!(
            lyric.lines,
            vec![
                (None, "Hello hi").into(),
                (None, "Bonjour salut").into(),
                (None, "おはよう こんにちは").into()
            ]
        );
    }
}
