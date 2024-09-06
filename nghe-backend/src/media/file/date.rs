use std::num::NonZeroU8;
use std::str::FromStr;

use time::macros::format_description;
use time::Month;

use crate::Error;

type FormatDescription<'a> = &'a [time::format_description::BorrowedFormatItem<'a>];

const YMD_MINUS_FORMAT: FormatDescription = format_description!("[year]-[month]-[day]");
const YM_MINUS_FORMAT: FormatDescription = format_description!("[year]-[month]");
const YMD_SLASH_FORMAT: FormatDescription = format_description!("[year]/[month]/[day]");
const YM_SLASH_FORMAT: FormatDescription = format_description!("[year]/[month]");
const YMD_DOT_FORMAT: FormatDescription = format_description!("[year].[month].[day]");
const YM_DOT_FORMAT: FormatDescription = format_description!("[year].[month]");
const Y_FORMAT: FormatDescription = format_description!("[year]");

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<Month>,
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
                return Err(Error::MediaDateFormat(s.to_owned()));
            }
        }

        Ok(Self { year: parsed.year(), month: parsed.month(), day: parsed.day() })
    }
}

#[cfg(test)]
mod test {
    use std::fmt::{Display, Formatter};

    use fake::{Dummy, Fake, Faker};

    use super::*;

    impl Display for Date {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if let Some(year) = self.year {
                let mut result = format!("{year:04}");
                if let Some(month) = self.month {
                    let month = month as u8;
                    result += &format!("-{month:02}");
                    if let Some(day) = self.day {
                        result += &format!("-{day:02}");
                    }
                }
                write!(f, "{result}")
            } else {
                Err(std::fmt::Error)
            }
        }
    }

    impl Dummy<Faker> for Date {
        fn dummy_with_rng<R: rand::Rng + ?Sized>(config: &Faker, rng: &mut R) -> Self {
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
}

#[cfg(test)]
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
