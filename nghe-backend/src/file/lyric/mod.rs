use core::str;
use std::borrow::Cow;

use alrc::AdvancedLrc;
use diesel::{ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use isolang::Language;
use lofty::id3::v2::{BinaryFrame, SynchronizedTextFrame, UnsynchronizedTextFrame};
use typed_path::Utf8TypedPath;
use uuid::Uuid;

use crate::database::Database;
use crate::filesystem::Trait as _;
use crate::orm::lyrics;
use crate::orm::upsert::Insert;
use crate::{Error, error, filesystem};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Clone))]
pub enum Lines<'a> {
    Unsync(Vec<Cow<'a, str>>),
    Sync(Vec<(u32, Cow<'a, str>)>),
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq, Clone))]
pub struct Lyric<'a> {
    pub description: Option<Cow<'a, str>>,
    pub language: Language,
    pub lines: Lines<'a>,
}

impl<'a> FromIterator<&'a str> for Lines<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::Unsync(iter.into_iter().map(Cow::Borrowed).collect())
    }
}

impl FromIterator<(u32, String)> for Lines<'_> {
    fn from_iter<T: IntoIterator<Item = (u32, String)>>(iter: T) -> Self {
        Self::Sync(iter.into_iter().map(|(duration, text)| (duration, text.into())).collect())
    }
}

impl<'a> TryFrom<&'a UnsynchronizedTextFrame<'_>> for Lyric<'a> {
    type Error = Error;

    fn try_from(frame: &'a UnsynchronizedTextFrame<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            description: Some(frame.description.as_str().into()),
            language: str::from_utf8(&frame.language)?.parse().map_err(error::Kind::from)?,
            lines: frame.content.lines().collect(),
        })
    }
}

impl<'a> TryFrom<&'a BinaryFrame<'_>> for Lyric<'a> {
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

impl<'a> Lyric<'a> {
    pub fn from_unsync_text(content: &'a str) -> Self {
        Self {
            description: None,
            language: Language::Und,
            lines: content.lines().filter(|text| !text.is_empty()).collect(),
        }
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
                    let minutes: u32 = line.time.minutes.into();
                    let seconds: u32 = line.time.seconds.into();
                    let milliseconds: u32 = line.time.millis.unwrap_or_default().into();
                    (minutes * 60 * 1000 + seconds * 1000 + milliseconds * 10, line.text)
                })
                .collect(),
        })
    }
}

impl Lyric<'_> {
    pub const EXTERNAL_EXTENSION: &'static str = "lrc";

    pub async fn upsert(
        &self,
        database: &Database,
        foreign: lyrics::Foreign,
        source: Option<impl AsRef<str>>,
    ) -> Result<Uuid, Error> {
        lyrics::Upsert {
            foreign,
            source: source.as_ref().map(AsRef::as_ref).map(Cow::Borrowed),
            data: self.try_into()?,
        }
        .insert(database)
        .await
    }

    async fn set_source_scanned_at(
        database: &Database,
        song_id: Uuid,
        source: impl AsRef<str>,
    ) -> Result<Option<Uuid>, Error> {
        diesel::update(lyrics::table)
            .filter(lyrics::song_id.eq(song_id))
            .filter(lyrics::source.eq(source.as_ref()))
            .set(lyrics::scanned_at.eq(crate::time::now().await))
            .returning(lyrics::id)
            .get_result(&mut database.get().await?)
            .await
            .optional()
            .map_err(Error::from)
    }

    pub async fn load(
        filesystem: &filesystem::Impl<'_>,
        path: Utf8TypedPath<'_>,
    ) -> Result<Option<Self>, Error> {
        if filesystem.exists(path).await? {
            let content = filesystem.read_to_string(path).await?;
            return Ok(Some(Self::from_sync_text(&content)?));
        }
        Ok(None)
    }

    pub async fn scan(
        database: &Database,
        filesystem: &filesystem::Impl<'_>,
        full: bool,
        song_id: Uuid,
        song_path: Utf8TypedPath<'_>,
    ) -> Result<Option<Uuid>, Error> {
        let path = song_path.with_extension(Self::EXTERNAL_EXTENSION);
        let path = path.to_path();

        Ok(
            if !full
                && let Some(lyrics_id) =
                    Self::set_source_scanned_at(database, song_id, path).await?
            {
                Some(lyrics_id)
            } else if let Some(lyrics) = Self::load(filesystem, path).await? {
                Some(lyrics.upsert(database, lyrics::Foreign { song_id }, Some(path)).await?)
            } else {
                None
            },
        )
    }

    pub async fn cleanup_one_external(
        database: &Database,
        started_at: time::OffsetDateTime,
        song_id: Uuid,
    ) -> Result<(), Error> {
        // Delete all lyrics of a song which haven't been refreshed since timestamp.
        diesel::delete(lyrics::table)
            .filter(lyrics::song_id.eq(song_id))
            .filter(lyrics::scanned_at.lt(started_at))
            .filter(lyrics::source.is_not_null())
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use std::fmt::Display;

    use diesel::{QueryDsl, SelectableHelper};
    use fake::{Dummy, Fake, Faker};
    use itertools::Itertools;

    use super::*;
    use crate::test::Mock;

    impl FromIterator<String> for Lines<'_> {
        fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
            Self::Unsync(iter.into_iter().map(Cow::Owned).collect())
        }
    }

    impl Lyric<'_> {
        pub fn is_sync(&self) -> bool {
            matches!(self.lines, Lines::Sync(_))
        }

        pub fn fake_sync() -> Self {
            Self {
                description: Faker.fake::<Option<String>>().map(Cow::Owned),
                language: Language::from_usize((0..=7915).fake()).unwrap(),
                lines: fake::vec![String; 1..=5]
                    .into_iter()
                    .map(|text| {
                        (
                            (((0..100).fake::<u32>() * 60 * 100
                                + (0..60).fake::<u32>() * 100
                                + (0..99).fake::<u32>())
                                * 10),
                            text,
                        )
                    })
                    .sorted()
                    .collect(),
            }
        }

        pub fn fake_unsync() -> Self {
            Self {
                description: None,
                language: Language::Und,
                lines: fake::vec![String; 1..=5].into_iter().collect(),
            }
        }
    }

    impl Lyric<'static> {
        pub async fn query_external(mock: &Mock, id: Uuid) -> Option<Self> {
            lyrics::table
                .filter(lyrics::song_id.eq(id))
                .filter(lyrics::source.is_not_null())
                .select(lyrics::Data::as_select())
                .get_result(&mut mock.get().await)
                .await
                .optional()
                .unwrap()
                .map(Self::from)
        }
    }

    impl Dummy<Faker> for Lyric<'_> {
        fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
            if config.fake_with_rng(rng) { Self::fake_sync() } else { Self::fake_unsync() }
        }
    }

    impl Display for Lyric<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.lines {
                Lines::Unsync(lines) => write!(f, "{}", lines.join("\n")),
                Lines::Sync(lines) => {
                    if let Some(description) = self.description.as_ref() {
                        writeln!(f, "[desc:{description}]")?;
                    }
                    writeln!(f, "[lang:{}]\n", self.language)?;

                    for (duration, text) in lines {
                        let seconds = duration / 1000;
                        let minutes = seconds / 60;
                        let seconds = seconds % 60;
                        let milliseconds = (duration % 1000) / 10;
                        write!(f, "[{minutes:02}:{seconds:02}.{milliseconds:02}]")?;
                        writeln!(f, "{text}")?;
                    }

                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test::assets;

    #[rstest]
    fn test_lyrics_roundtrip(#[values(true, false)] sync: bool) {
        if sync {
            let lyrics = Lyric::fake_sync();
            assert_eq!(lyrics, Lyric::from_sync_text(&lyrics.to_string()).unwrap());
        } else {
            let lyrics = Lyric::fake_unsync();
            assert_eq!(lyrics, Lyric::from_unsync_text(&lyrics.to_string()));
        }
    }

    #[rstest]
    #[case("sync.lrc", Lyric {
        description: Some("Lyric".to_owned().into()),
        language: Language::Eng,
        lines: vec![
            (1020_u32, "Hello hi".to_owned()),
            (3040_u32, "Bonjour salut".to_owned()),
            (5060_u32, "おはよう こんにちは".to_owned()),
        ]
        .into_iter()
        .collect()
    })]
    #[case("unsync.txt", Lyric {
        description: None,
        language: Language::Und,
        lines: vec![
            "Hello hi",
            "Bonjour salut",
            "おはよう こんにちは",
        ]
        .into_iter()
        .collect()
    })]
    fn test_from_text(#[case] filename: &str, #[case] lyrics: Lyric<'_>) {
        let content = std::fs::read_to_string(assets::dir().join("lyrics").join(filename)).unwrap();
        let parsed = if lyrics.is_sync() {
            Lyric::from_sync_text(&content).unwrap()
        } else {
            Lyric::from_unsync_text(&content)
        };
        assert_eq!(parsed, lyrics);
    }
}
