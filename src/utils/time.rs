use super::song::SongDate;

use anyhow::Result;
use time::{
    format_description::well_known::{iso8601, Iso8601},
    macros::format_description,
    parsing::Parsed,
    serde,
};

const ISO8601_CONFIG: iso8601::EncodedConfig = iso8601::Config::DEFAULT
    .set_year_is_six_digits(false)
    .encode();
const ISO8601_FORMAT: Iso8601<ISO8601_CONFIG> = Iso8601::<ISO8601_CONFIG>;
serde::format_description!(iso8601_datetime_format, OffsetDateTime, ISO8601_FORMAT);

pub mod iso8601_datetime {
    pub use super::iso8601_datetime_format::*;

    pub mod option {
        pub use super::super::iso8601_datetime_format::option::*;
    }
}

type FormatDescription<'a> = &'a [time::format_description::FormatItem<'a>];

const YMD_FORMAT: FormatDescription = format_description!("[year]-[month]-[day]");
const YM_FORMAT: FormatDescription = format_description!("[year]-[month]");
const Y_FORMAT: FormatDescription = format_description!("[year]");

pub fn parse_date(input: Option<&str>) -> Result<SongDate> {
    if let Some(input) = input {
        let mut parsed = Parsed::new();
        if input.len() >= 10 {
            // yyyy-mm-dd
            parsed.parse_items(input[..10].as_bytes(), YMD_FORMAT)?;
            let year = parsed.year().expect("error in time parsing") as u16;
            let month = parsed.month().expect("error in time parsing") as u8;
            let day = u8::from(parsed.day().expect("error in time parsing"));
            Ok(Some((year, Some((month, Some(day))))))
        } else if input.len() >= 7 {
            // yyyy-mm
            parsed.parse_items(input[..7].as_bytes(), YM_FORMAT)?;
            let year = parsed.year().expect("error in time parsing") as u16;
            let month = parsed.month().expect("error in time parsing") as u8;
            Ok(Some((year, Some((month, None)))))
        } else {
            // yyyy
            parsed.parse_items(input[..4].as_bytes(), Y_FORMAT)?;
            let year = parsed.year().expect("error in time parsing") as u16;
            Ok(Some((year, None)))
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let date = parse_date(None).unwrap();
        assert_eq!(date, None);

        let date = parse_date(Some("2000-12-01")).unwrap();
        assert_eq!(date, Some((2000, Some((12, Some(1))))));

        let date = parse_date(Some("2000-12-01-still-ok")).unwrap();
        assert_eq!(date, Some((2000, Some((12, Some(1))))));

        let date = parse_date(Some("2000-12")).unwrap();
        assert_eq!(date, Some((2000, Some((12, None)))));

        let date = parse_date(Some("2000")).unwrap();
        assert_eq!(date, Some((2000, None)));

        assert!(parse_date(Some("2000-31")).is_err());
        assert!(parse_date(Some("12-01")).is_err());
        assert!(parse_date(Some("invalid")).is_err());
    }
}
