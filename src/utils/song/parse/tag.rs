use anyhow::Result;
#[cfg(test)]
pub use fake::{Dummy, Fake};
use isolang::Language;
#[cfg(test)]
pub use itertools::Itertools;
use lofty::picture::Picture;
use time::macros::format_description;
use tracing::instrument;
use uuid::Uuid;

use crate::models::*;
use crate::OSError;

type FormatDescription<'a> = &'a [time::format_description::BorrowedFormatItem<'a>];

const YMD_MINUS_FORMAT: FormatDescription = format_description!("[year]-[month]-[day]");
const YM_MINUS_FORMAT: FormatDescription = format_description!("[year]-[month]");
const YMD_SLASH_FORMAT: FormatDescription = format_description!("[year]/[month]/[day]");
const YM_SLASH_FORMAT: FormatDescription = format_description!("[year]/[month]");
const YMD_DOT_FORMAT: FormatDescription = format_description!("[year].[month].[day]");
const YM_DOT_FORMAT: FormatDescription = format_description!("[year].[month]");
const Y_FORMAT: FormatDescription = format_description!("[year]");

type SongDateInner = Option<(u16, Option<(u8, Option<u8>)>)>;
#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(Default, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct SongDate(pub SongDateInner);

#[derive(Debug)]
#[cfg_attr(test, derive(Dummy, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord))]
pub struct MediaDateMbz {
    pub name: String,
    pub date: SongDate,
    pub release_date: SongDate,
    pub original_release_date: SongDate,
    pub mbz_id: Option<Uuid>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Dummy, Clone, PartialEq, Eq))]
pub struct SongTag {
    pub song: MediaDateMbz,
    pub album: MediaDateMbz,

    #[cfg_attr(
        test,
        dummy(
            expr = "artists::ArtistNoId::fake_vec(1..=5).into_iter().unique().sorted().collect()"
        )
    )]
    pub artists: Vec<artists::ArtistNoId>,
    #[cfg_attr(
        test,
        dummy(
            expr = "artists::ArtistNoId::fake_vec(0..=5).into_iter().unique().sorted().collect()"
        )
    )]
    pub album_artists: Vec<artists::ArtistNoId>,

    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,

    #[cfg_attr(
        test,
        dummy(expr = "((0..=7915), 0..=2).fake::<Vec<usize>>().into_iter().unique().map(|i| \
                      Language::from_usize(i).unwrap()).sorted().collect()")
    )]
    pub languages: Vec<Language>,
    #[cfg_attr(
        test,
        dummy(expr = "fake::vec![String; 0..=2].into_iter().map(genres::Genre::from).collect()")
    )]
    pub genres: Vec<genres::Genre>,
    pub compilation: bool,

    #[cfg_attr(test, dummy(expr = "crate::utils::test::picture::fake(false)"))]
    pub picture: Option<Picture>,
}

impl MediaDateMbz {
    pub fn date_or_default(&self) -> SongDate {
        self.date.or(self.original_release_date).or(self.release_date)
    }

    pub fn release_date_or_default(&self) -> SongDate {
        self.release_date.or(self.date)
    }
}

impl SongTag {
    pub fn album_artists_or_default(&self) -> &Vec<artists::ArtistNoId> {
        if !self.album_artists.is_empty() { &self.album_artists } else { &self.artists }
    }
}

impl<'a> From<&'a MediaDateMbz> for albums::NewAlbum<'a> {
    fn from(value: &'a MediaDateMbz) -> Self {
        Self {
            name: (&value.name).into(),
            date: value.date_or_default().into(),
            release_date: value.release_date_or_default().into(),
            original_release_date: value.original_release_date.into(),
            mbz_id: value.mbz_id,
        }
    }
}

impl SongDate {
    #[instrument(err(Debug))]
    pub fn parse(input: Option<&str>) -> Result<Self> {
        if let Some(input) = input
            && input.len() >= 4
        {
            let mut parsed = time::parsing::Parsed::new();
            if input.len() >= 10 {
                // yyyy-mm-dd
                let input = input[..10].as_bytes();
                if parsed.parse_items(input, YMD_MINUS_FORMAT).is_ok()
                    || parsed.parse_items(input, YMD_SLASH_FORMAT).is_ok()
                    || parsed.parse_items(input, YMD_DOT_FORMAT).is_ok()
                {
                    let year = parsed.year().ok_or_else(|| OSError::NotFound("year".into()))? as _;
                    let month =
                        parsed.month().ok_or_else(|| OSError::NotFound("month".into()))? as _;
                    let day = parsed.day().ok_or_else(|| OSError::NotFound("day".into()))?.get();
                    Ok(Self(Some((year, Some((month, Some(day)))))))
                } else {
                    anyhow::bail!(OSError::InvalidParameter("can not parse date input".into()))
                }
            } else if input.len() >= 7 {
                // yyyy-mm
                let input = input[..7].as_bytes();
                if parsed.parse_items(input, YM_MINUS_FORMAT).is_ok()
                    || parsed.parse_items(input, YM_SLASH_FORMAT).is_ok()
                    || parsed.parse_items(input, YM_DOT_FORMAT).is_ok()
                {
                    let year = parsed.year().ok_or_else(|| OSError::NotFound("year".into()))? as _;
                    let month =
                        parsed.month().ok_or_else(|| OSError::NotFound("month".into()))? as _;
                    Ok(Self(Some((year, Some((month, None))))))
                } else {
                    anyhow::bail!(OSError::InvalidParameter("can not parse date input".into()))
                }
            } else {
                // yyyy
                let input = input[..4].as_bytes();
                if parsed.parse_items(input, Y_FORMAT).is_ok() {
                    let year = parsed.year().ok_or_else(|| OSError::NotFound("year".into()))? as _;
                    Ok(Self(Some((year, None))))
                } else {
                    anyhow::bail!(OSError::InvalidParameter("can not parse date input".into()))
                }
            }
        } else {
            Ok(Self(None))
        }
    }

    pub fn or(self, date: Self) -> Self {
        if self.0.is_some() { self } else { date }
    }

    pub fn to_ymd(self) -> (Option<i16>, Option<i16>, Option<i16>) {
        if let Some((year, remainder)) = self.0 {
            let year = year as _;
            if let Some((month, remainder)) = remainder {
                let month = month as _;
                if let Some(day) = remainder {
                    let day = day as _;
                    (Some(year), Some(month), Some(day))
                } else {
                    (Some(year), Some(month), None)
                }
            } else {
                (Some(year), None, None)
            }
        } else {
            (None, None, None)
        }
    }
}

#[cfg(test)]
pub mod test {
    use fake::Faker;

    use super::*;
    use crate::utils::song::SongInformation;

    impl From<String> for MediaDateMbz {
        fn from(value: String) -> Self {
            Self { name: value, ..Default::default() }
        }
    }

    impl From<&str> for MediaDateMbz {
        fn from(value: &str) -> Self {
            value.to_string().into()
        }
    }

    impl From<MediaDateMbz> for albums::NewAlbum<'static> {
        fn from(value: MediaDateMbz) -> Self {
            Self {
                date: value.date_or_default().into(),
                release_date: value.release_date_or_default().into(),
                original_release_date: value.original_release_date.into(),
                mbz_id: value.mbz_id,
                name: value.name.into(),
            }
        }
    }

    impl SongTag {
        pub fn to_information(&self) -> SongInformation {
            SongInformation { tag: self.clone(), property: Default::default(), lrc: None }
        }
    }

    impl SongDate {
        pub fn from_ymd(year: Option<i16>, month: Option<i16>, day: Option<i16>) -> Self {
            if let Some(year) = year {
                let year = year as _;
                if let Some(month) = month {
                    let month = month as _;
                    if let Some(day) = day {
                        let day = day as _;
                        Self(Some((year, Some((month, Some(day))))))
                    } else {
                        Self(Some((year, Some((month, None)))))
                    }
                } else {
                    Self(Some((year, None)))
                }
            } else {
                Self(None)
            }
        }

        pub fn to_string(&self) -> Option<String> {
            if let Some((year, remainder)) = self.0 {
                let mut result = format!("{:04}", year);
                if let Some((month, remainder)) = remainder {
                    result = format!("{}-{:02}", result, month);
                    if let Some(day) = remainder {
                        result = format!("{}-{:02}", result, day);
                    }
                }
                Some(result)
            } else {
                None
            }
        }
    }

    impl Dummy<Faker> for SongDate {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            let date: time::Date = Fake::fake_with_rng(&Faker, rng);
            let year = if Fake::fake_with_rng(&Faker, rng) {
                Some(date.year().clamp(0, 9999) as _)
            } else {
                None
            };
            let month =
                if Fake::fake_with_rng(&Faker, rng) { Some(date.month() as _) } else { None };
            let day = if Fake::fake_with_rng(&Faker, rng) { Some(date.day() as _) } else { None };
            Self::from_ymd(year, month, day)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_song_date() {
        let date = SongDate::parse(None).unwrap();
        assert_eq!(date.0, None);

        let date = SongDate::parse(Some("2000-12-01")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, Some(1))))));
        let date = SongDate::parse(Some("2000/12/01")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, Some(1))))));
        let date = SongDate::parse(Some("2000.12.01")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, Some(1))))));

        let date = SongDate::parse(Some("2000-12-01-still-ok")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, Some(1))))));
        let date = SongDate::parse(Some("2000/12/01 still ok")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, Some(1))))));
        let date = SongDate::parse(Some("2000.12.01:still:ok")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, Some(1))))));

        let date = SongDate::parse(Some("2000-12")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, None)))));
        let date = SongDate::parse(Some("2000/12")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, None)))));
        let date = SongDate::parse(Some("2000.12")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, None)))));

        let date = SongDate::parse(Some("2000")).unwrap();
        assert_eq!(date.0, Some((2000, None)));

        let date = SongDate::parse(Some("")).unwrap();
        assert_eq!(date.0, None);

        assert!(SongDate::parse(Some("2000-31")).is_err());
        assert!(SongDate::parse(Some("12-01")).is_err());
        assert!(SongDate::parse(Some("invalid")).is_err());
    }
}
