use core::str;
use std::borrow::Cow;

use alrc::AdvancedLrc;
use diesel::{ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use futures_lite::{StreamExt as _, stream};
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
        let description = frame.description.as_str();
        Ok(Self {
            description: if description.is_empty() { None } else { Some(description.into()) },
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
        let lines = content.lines().filter(|text| !text.is_empty());
        let (description, lines) = if cfg!(test) && content.starts_with('#') {
            let mut lines = lines;
            let description = lines.next().unwrap().strip_prefix('#').unwrap();
            (if description.is_empty() { None } else { Some(description.into()) }, lines.collect())
        } else {
            (None, lines.collect())
        };
        Self { description, language: Language::Und, lines }
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
        external: bool,
    ) -> Result<Uuid, Error> {
        lyrics::Upsert { foreign, external, data: self.try_into()? }.insert(database).await
    }

    pub async fn upserts_embedded(
        database: &Database,
        foreign: lyrics::Foreign,
        lyrics: &[Self],
    ) -> Result<Vec<Uuid>, Error> {
        stream::iter(lyrics)
            .then(async |lyric| lyric.upsert(database, foreign, false).await)
            .try_collect()
            .await
    }

    async fn set_external_scanned_at(
        database: &Database,
        song_id: Uuid,
    ) -> Result<Option<Uuid>, Error> {
        diesel::update(lyrics::table)
            .filter(lyrics::song_id.eq(song_id))
            .filter(lyrics::external)
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
        Ok(
            if !full
                && let Some(lyrics_id) = Self::set_external_scanned_at(database, song_id).await?
            {
                Some(lyrics_id)
            } else if let Some(lyrics) =
                Self::load(filesystem, song_path.with_extension(Self::EXTERNAL_EXTENSION).to_path())
                    .await?
            {
                Some(lyrics.upsert(database, lyrics::Foreign { song_id }, true).await?)
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
            .filter(lyrics::external)
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }

    pub async fn cleanup_one(
        database: &Database,
        started_at: time::OffsetDateTime,
        song_id: Uuid,
    ) -> Result<(), Error> {
        // Delete all lyrics of a song which haven't been refreshed since timestamp.
        diesel::delete(lyrics::table)
            .filter(lyrics::song_id.eq(song_id))
            .filter(lyrics::scanned_at.lt(started_at))
            .execute(&mut database.get().await?)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use std::fmt::Display;

    use diesel::dsl::not;
    use diesel::{QueryDsl, SelectableHelper};
    use fake::{Dummy, Fake, Faker};
    use itertools::Itertools;
    use lofty::id3::v2::Frame;

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
            // Force description as Some to avoid clash with unsync.
            Self {
                description: Some(Faker.fake::<String>().into()),
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
                description: Faker.fake::<Option<String>>().map(Cow::Owned),
                language: Language::Und,
                lines: fake::vec![String; 1..=5].into_iter().collect(),
            }
        }

        pub fn fake_vec() -> Vec<Self> {
            let unsync = if Faker.fake() { Some(Self::fake_unsync()) } else { None };
            let sync = if Faker.fake() { Some(Self::fake_sync()) } else { None };
            unsync.into_iter().chain(sync).collect()
        }
    }

    impl Lyric<'static> {
        pub async fn query(mock: &Mock, id: Uuid) -> Self {
            lyrics::table
                .filter(lyrics::id.eq(id))
                .select(lyrics::Data::as_select())
                .get_result(&mut mock.get().await)
                .await
                .unwrap()
                .into()
        }

        pub async fn query_embedded(mock: &Mock, id: Uuid) -> Vec<Self> {
            lyrics::table
                .filter(lyrics::song_id.eq(id))
                .filter(not(lyrics::external))
                .select(lyrics::Data::as_select())
                .order_by(lyrics::scanned_at)
                .get_results(&mut mock.get().await)
                .await
                .unwrap()
                .into_iter()
                .map(Self::from)
                .collect()
        }

        pub async fn query_external(mock: &Mock, id: Uuid) -> Option<Self> {
            lyrics::table
                .filter(lyrics::song_id.eq(id))
                .filter(lyrics::external)
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
                Lines::Unsync(lines) => {
                    if let Some(description) = self.description.as_ref() {
                        writeln!(f, "#{description}")?;
                    }
                    write!(f, "{}", lines.join("\n"))?;
                }
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
                }
            }
            Ok(())
        }
    }

    impl From<Lyric<'_>> for Frame<'static> {
        fn from(value: Lyric<'_>) -> Self {
            let language = value.language.to_639_3().as_bytes().try_into().unwrap();
            match value.lines {
                Lines::Unsync(lines) => UnsynchronizedTextFrame::new(
                    lofty::TextEncoding::UTF8,
                    language,
                    value.description.map(Cow::into_owned).unwrap_or_default(),
                    lines.join("\n"),
                )
                .into(),
                Lines::Sync(lines) => BinaryFrame::new(
                    crate::config::parsing::id3v2::Id3v2::SYNC_LYRIC_FRAME_ID,
                    SynchronizedTextFrame::new(
                        lofty::TextEncoding::UTF8,
                        language,
                        lofty::id3::v2::TimestampFormat::MS,
                        lofty::id3::v2::SyncTextContentType::Lyrics,
                        value.description.map(Cow::into_owned),
                        lines
                            .into_iter()
                            .map(|(duration, text)| (duration, text.into_owned()))
                            .collect(),
                    )
                    .as_bytes()
                    .unwrap(),
                )
                .into(),
            }
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::test::filesystem::Trait as _;
    use crate::test::{Mock, assets, mock};

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

    #[rstest]
    #[tokio::test]
    async fn test_lyric_upsert_roundtrip(
        #[future(awt)] mock: Mock,
        #[values(true, false)] external: bool,
        #[values(true, false)] update_lyric: bool,
        #[values(true, false)] same_description: bool,
    ) {
        let mut music_folder = mock.music_folder(0).await;
        let song_id = music_folder.add_audio().call().await.song_id(0);

        let lyric: Lyric = Faker.fake();
        let id =
            lyric.upsert(mock.database(), lyrics::Foreign { song_id }, external).await.unwrap();

        let database_lyric = Lyric::query(&mock, id).await;
        assert_eq!(database_lyric, lyric);

        if update_lyric {
            let update_lyric = Lyric {
                description: if same_description {
                    lyric.description.clone()
                } else {
                    // Force description to Some to avoid both descriptions are None.
                    Some(Faker.fake::<String>().into())
                },
                ..Faker.fake()
            };
            let update_id = update_lyric
                .upsert(mock.database(), lyrics::Foreign { song_id }, external)
                .await
                .unwrap();
            let database_update_lyric = Lyric::query(&mock, id).await;

            if external || same_description {
                assert_eq!(id, update_id);
                assert_eq!(database_update_lyric, update_lyric);
            } else {
                // This will always insert a new row to the database
                // since there is nothing to identify the old lyric.
                assert_ne!(id, update_id);
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_scan_full(
        #[future(awt)]
        #[with(0, 1)]
        mock: Mock,
        #[values(true, false)] full: bool,
    ) {
        let mut music_folder = mock.music_folder(0).await;
        let song_id = music_folder.add_audio().call().await.song_id(0);

        let lyric: Lyric = Faker.fake();
        let id = lyric.upsert(mock.database(), lyrics::Foreign { song_id }, true).await.unwrap();

        let new_lyric = Lyric::fake_sync();

        let filesystem = music_folder.to_impl();
        let path = filesystem.prefix().join("test");
        let path = path.to_path();
        filesystem
            .write(
                path.with_extension(Lyric::EXTERNAL_EXTENSION).to_path(),
                new_lyric.to_string().as_bytes(),
            )
            .await;
        let scanned_id = Lyric::scan(mock.database(), &filesystem.main(), full, song_id, path)
            .await
            .unwrap()
            .unwrap();

        // They will always be the same because there can only be at most one external lyric for a
        // song.
        assert_eq!(scanned_id, id);
        let database_lyric = Lyric::query(&mock, id).await;
        assert_eq!(database_lyric, if full { new_lyric } else { lyric });
    }
}
