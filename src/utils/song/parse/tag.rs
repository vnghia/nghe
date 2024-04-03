use anyhow::{Context, Result};
use concat_string::concat_string;
#[cfg(test)]
pub use fake::{Dummy, Fake};
use isolang::Language;
#[cfg(test)]
pub use itertools::Itertools;
use lofty::Picture;
use time::macros::format_description;

type SongDateInner = Option<(u16, Option<(u8, Option<u8>)>)>;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct SongDate(pub SongDateInner);

type FormatDescription<'a> = &'a [time::format_description::FormatItem<'a>];

const YMD_FORMAT: FormatDescription = format_description!("[year]-[month]-[day]");
const YM_FORMAT: FormatDescription = format_description!("[year]-[month]");
const Y_FORMAT: FormatDescription = format_description!("[year]");

#[derive(Debug)]
#[cfg_attr(test, derive(Dummy, Clone, PartialEq, Eq))]
pub struct SongTag {
    pub title: String,
    pub album: String,
    #[cfg_attr(
        test,
        dummy(expr = "fake::vec![String; 1..=5].into_iter().unique().sorted().collect()")
    )]
    pub artists: Vec<String>,
    #[cfg_attr(
        test,
        dummy(expr = "fake::vec![String; 0..=5].into_iter().unique().sorted().collect()")
    )]
    pub album_artists: Vec<String>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    pub date: SongDate,
    pub release_date: SongDate,
    pub original_release_date: SongDate,
    #[cfg_attr(
        test,
        dummy(expr = "((0..=7915), 0..=2).fake::<Vec<usize>>().into_iter().unique().map(|i| \
                      Language::from_usize(i).unwrap()).sorted().collect()")
    )]
    pub languages: Vec<Language>,
    #[cfg_attr(test, dummy(expr = "crate::utils::test::picture::fake(false)"))]
    pub picture: Option<Picture>,
}

impl SongTag {
    pub fn album_artists_or_default(&self) -> &Vec<String> {
        if !self.album_artists.is_empty() { &self.album_artists } else { &self.artists }
    }

    pub fn date_or_default(&self) -> SongDate {
        self.date.or(self.original_release_date).or(self.release_date)
    }

    pub fn release_date_or_default(&self) -> SongDate {
        self.release_date.or(self.date)
    }
}

impl SongDate {
    pub fn parse(input: Option<&str>) -> Result<Self> {
        if let Some(input) = input {
            let mut parsed = time::parsing::Parsed::new();
            if input.len() >= 10 {
                // yyyy-mm-dd
                parsed
                    .parse_items(input[..10].as_bytes(), YMD_FORMAT)
                    .with_context(|| concat_string!("date value: ", input))?;
                let year = parsed.year().expect("error in time parsing") as _;
                let month = parsed.month().expect("error in time parsing") as _;
                let day = u8::from(parsed.day().expect("error in time parsing"));
                Ok(Self(Some((year, Some((month, Some(day)))))))
            } else if input.len() >= 7 {
                // yyyy-mm
                parsed
                    .parse_items(input[..7].as_bytes(), YM_FORMAT)
                    .with_context(|| concat_string!("date value: ", input))?;
                let year = parsed.year().expect("error in time parsing") as _;
                let month = parsed.month().expect("error in time parsing") as _;
                Ok(Self(Some((year, Some((month, None))))))
            } else {
                // yyyy
                parsed
                    .parse_items(input[..4].as_bytes(), Y_FORMAT)
                    .with_context(|| concat_string!("date value: ", input))?;
                let year = parsed.year().expect("error in time parsing") as _;
                Ok(Self(Some((year, None))))
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
    use crate::open_subsonic::test::id3::db::*;
    use crate::utils::song::SongInformation;

    impl SongTag {
        pub fn to_information(&self) -> SongInformation {
            SongInformation { tag: self.clone(), property: Default::default() }
        }
    }

    impl SongDate {
        pub fn from_id3_db(db: DateId3Db) -> Self {
            Self::from_ymd(db.year, db.month, db.day)
        }

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
                Some(date.year().min(9999).max(0) as _)
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

        let date = SongDate::parse(Some("2000-12-01-still-ok")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, Some(1))))));

        let date = SongDate::parse(Some("2000-12")).unwrap();
        assert_eq!(date.0, Some((2000, Some((12, None)))));

        let date = SongDate::parse(Some("2000")).unwrap();
        assert_eq!(date.0, Some((2000, None)));

        assert!(SongDate::parse(Some("2000-31")).is_err());
        assert!(SongDate::parse(Some("12-01")).is_err());
        assert!(SongDate::parse(Some("invalid")).is_err());
    }
}
