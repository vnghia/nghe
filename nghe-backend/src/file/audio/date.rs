use std::num::NonZeroU8;
use std::str::FromStr;

use o2o::o2o;
use time::Month;
use time::macros::format_description;

use crate::orm::{albums, songs};
use crate::{Error, error};

type FormatDescription<'a> = &'a [time::format_description::BorrowedFormatItem<'a>];

const YMD_MINUS_FORMAT: FormatDescription = format_description!("[year]-[month]-[day]");
const YM_MINUS_FORMAT: FormatDescription = format_description!("[year]-[month]");
const YMD_SLASH_FORMAT: FormatDescription = format_description!("[year]/[month]/[day]");
const YM_SLASH_FORMAT: FormatDescription = format_description!("[year]/[month]");
const YMD_DOT_FORMAT: FormatDescription = format_description!("[year].[month].[day]");
const YM_DOT_FORMAT: FormatDescription = format_description!("[year].[month]");
const Y_FORMAT: FormatDescription = format_description!("[year]");

#[derive(Debug, Default, Clone, Copy, o2o)]
#[try_map_owned(songs::date::Date, Error)]
#[try_map_owned(songs::date::Release, Error)]
#[try_map_owned(songs::date::OriginalRelease, Error)]
#[try_map_owned(albums::date::Date, Error)]
#[try_map_owned(albums::date::Release, Error)]
#[try_map_owned(albums::date::OriginalRelease, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Date {
    #[from(~.map(i32::from))]
    #[into(~.map(i32::try_into).transpose()?)]
    pub year: Option<i32>,
    #[from(~.map(u8::try_from).transpose()?.map(Month::try_from).transpose()?)]
    #[into(~.map(|month| (month as u8).into()))]
    pub month: Option<Month>,
    #[from(~.map(u8::try_from).transpose()?.map(NonZeroU8::try_from).transpose()?)]
    #[into(~.map(NonZeroU8::get).map(u8::into))]
    pub day: Option<NonZeroU8>,
}

impl Date {
    pub fn is_some(&self) -> bool {
        self.year.is_some()
    }
}

impl FromStr for Date {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parsed = time::parsing::Parsed::new();
        let input = s.as_bytes();
        if !s.is_empty()
            && parsed.parse_items(input, YMD_MINUS_FORMAT).is_err()
            && parsed.parse_items(input, YMD_SLASH_FORMAT).is_err()
            && parsed.parse_items(input, YMD_DOT_FORMAT).is_err()
            && parsed.parse_items(input, YM_MINUS_FORMAT).is_err()
            && parsed.parse_items(input, YM_SLASH_FORMAT).is_err()
            && parsed.parse_items(input, YM_DOT_FORMAT).is_err()
        {
            // Don't aggresively parse everything as year
            let result = parsed.parse_items(input, Y_FORMAT);
            if result.is_err() || result.is_ok_and(|remain| !remain.is_empty()) {
                return error::Kind::InvalidDateTagFormat(s.to_owned()).into();
            }
        }

        Ok(Self { year: parsed.year(), month: parsed.month(), day: parsed.day() })
    }
}

impl TryFrom<&lofty::tag::items::Timestamp> for Date {
    type Error = Error;

    fn try_from(value: &lofty::tag::items::Timestamp) -> Result<Self, Self::Error> {
        Ok(Self {
            year: Some(value.year.into()),
            month: value.month.map(Month::try_from).transpose()?,
            day: value.day.map(NonZeroU8::try_from).transpose()?,
        })
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use std::fmt::{Display, Formatter, Write};

    use fake::{Dummy, Fake, Faker};

    use super::*;

    impl Display for Date {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if let Some(year) = self.year {
                let mut result = format!("{year:04}");
                if let Some(month) = self.month {
                    let month = month as u8;
                    write!(result, "-{month:02}")?;
                    if let Some(day) = self.day {
                        write!(result, "-{day:02}")?;
                    }
                }
                write!(f, "{result}")
            } else {
                Err(std::fmt::Error)
            }
        }
    }

    impl Dummy<Faker> for Date {
        fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
            let date: time::Date = config.fake_with_rng(rng);

            let year =
                if config.fake_with_rng(rng) { Some(date.year().clamp(0, 9999)) } else { None };
            let month =
                if year.is_some() && config.fake_with_rng(rng) { Some(date.month()) } else { None };
            let day = if month.is_some() && config.fake_with_rng(rng) {
                Some(date.day().try_into().unwrap())
            } else {
                None
            };

            Self { year, month, day }
        }
    }

    impl From<Date> for Option<lofty::tag::items::Timestamp> {
        fn from(value: Date) -> Self {
            value.year.map(|year| lofty::tag::items::Timestamp {
                year: year.try_into().unwrap(),
                month: value.month.map(u8::try_from).transpose().unwrap(),
                day: value.day.map(NonZeroU8::get),
                ..Default::default()
            })
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("2000-12-01", Some(2000), Some(Month::December), Some(1))]
    #[case("2000/12/01", Some(2000), Some(Month::December), Some(1))]
    #[case("2000.12.01", Some(2000), Some(Month::December), Some(1))]
    #[case("2000-12-01-still-ok", Some(2000), Some(Month::December), Some(1))]
    #[case("2000.12.01/still/ok", Some(2000), Some(Month::December), Some(1))]
    #[case("2000-12-01T02:29:01Z", Some(2000), Some(Month::December), Some(1))]
    #[case("2000-12", Some(2000), Some(Month::December), None)]
    #[case("2000/12", Some(2000), Some(Month::December), None)]
    #[case("2000.12", Some(2000), Some(Month::December), None)]
    #[case("2000", Some(2000), None, None)]
    #[case("", None, None, None)]
    fn test_parse_success(
        #[case] input: &'static str,
        #[case] year: Option<i32>,
        #[case] month: Option<Month>,
        #[case] day: Option<u8>,
    ) {
        let date: Date = input.parse().unwrap();
        assert_eq!(date.year, year);
        assert_eq!(date.month, month);
        assert_eq!(date.day.map(NonZeroU8::get), day);
    }

    #[rstest]
    #[case("2000-31")]
    #[case("20-12-01")]
    #[case("invalid")]
    #[case("12-01")]
    #[case("31")]
    fn test_parse_error(#[case] input: &'static str) {
        assert!(input.parse::<Date>().is_err());
    }

    #[rstest]
    #[case(Some(2000), Some(Month::December), Some(31), "2000-12-31")]
    #[case(Some(2000), Some(Month::December), None, "2000-12")]
    #[case(Some(2000), None, Some(31), "2000")]
    #[case(Some(2000), None, None, "2000")]
    fn test_display(
        #[case] year: Option<i32>,
        #[case] month: Option<Month>,
        #[case] day: Option<u8>,
        #[case] str: &str,
    ) {
        assert_eq!(
            Date { year, month, day: day.map(u8::try_into).transpose().unwrap() }.to_string(),
            str
        );
    }
}
