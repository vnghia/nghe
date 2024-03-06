use crate::OSError;

use anyhow::Result;
use isolang::Language;
use lofty::Accessor;

pub type SongDate = Option<(u16, Option<(u8, Option<u8>)>)>;

#[cfg(test)]
pub use fake::{Dummy, Fake, Faker, Opt, Optional};

#[derive(Debug)]
#[cfg_attr(test, derive(Dummy, Clone, PartialEq, Eq))]
pub struct SongTag {
    pub title: String,
    pub album: String,
    #[cfg_attr(test, dummy(faker = "(Faker, 1..2)"))]
    pub artists: Vec<String>,
    pub album_artists: Vec<String>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    #[cfg_attr(
        test,
        dummy(
            expr = "Opt((0..9999, Opt((1..12, Opt(1..28, 50)), 50)), 50).fake::<Optional<(u16, Optional<(u8, Optional<u8>)>)>>().0.map(|y| (y.0, y.1.0.map(|m| (m.0, m.1.0))))"
        )
    )]
    pub date: SongDate,
    #[cfg_attr(test, dummy(expr = "None"))]
    pub release_date: SongDate,
    #[cfg_attr(
        test,
        dummy(
            expr = "Opt((0..9999, Opt((1..12, Opt(1..28, 50)), 50)), 50).fake::<Optional<(u16, Optional<(u8, Optional<u8>)>)>>().0.map(|y| (y.0, y.1.0.map(|m| (m.0, m.1.0))))"
        )
    )]
    pub original_release_date: SongDate,
    #[cfg_attr(
        test,
        dummy(
            expr = "((0..=7915), 0..=2).fake::<Vec<usize>>().into_iter().map(|i| Language::from_usize(i).unwrap()).collect()"
        )
    )]
    pub languages: Vec<Language>,
}

impl SongTag {
    pub fn album_artists_or_default(&self) -> &Vec<String> {
        if !self.album_artists.is_empty() {
            &self.album_artists
        } else {
            &self.artists
        }
    }

    pub fn date_or_default(&self) -> SongDate {
        self.date
            .or(self.original_release_date)
            .or(self.release_date)
    }

    pub fn release_date_or_default(&self) -> SongDate {
        self.release_date.or(self.date)
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(Default))]
pub struct SongProperty {
    pub duration: f32,
}

pub fn extract_common_tags<T: Accessor>(tag: &mut T) -> Result<(String, String)> {
    let title = tag
        .title()
        .ok_or_else(|| OSError::NotFound("Title tag".into()))?
        .into_owned();
    let album = tag
        .album()
        .ok_or_else(|| OSError::NotFound("Album tag".into()))?
        .into_owned();
    Ok((title, album))
}

pub fn parse_number_and_total(
    number_value: Option<&str>,
    total_value: Option<&str>,
) -> Result<(Option<u32>, Option<u32>)> {
    if let Some(number_value) = number_value {
        if let Some((number_value, total_value)) = number_value.split_once('/') {
            // mpeg tag does not have a separate value for total value.
            // therefore if total value is present and number value is not,
            // a negative value will be written to number value.
            Ok((
                if !cfg!(test) {
                    Some(number_value.parse()?)
                } else {
                    number_value.parse().ok()
                },
                Some(total_value.parse()?),
            ))
        } else {
            Ok((
                Some(number_value.parse()?),
                total_value.map(|v| v.parse()).transpose()?,
            ))
        }
    } else {
        Ok((None, total_value.map(|v| v.parse()).transpose()?))
    }
}

pub fn split_song_date(song_date: SongDate) -> (Option<i16>, Option<i16>, Option<i16>) {
    if let Some((year, remainder)) = song_date {
        let year = year as i16;
        if let Some((month, remainder)) = remainder {
            let month = month as i16;
            if let Some(day) = remainder {
                let day = day as i16;
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

#[cfg(test)]
pub mod test {
    use crate::utils::song::SongInformation;

    impl super::SongTag {
        pub fn to_information(&self) -> SongInformation {
            SongInformation {
                tag: self.clone(),
                property: Default::default(),
            }
        }
    }

    pub fn song_date_to_string(song_date: &super::SongDate) -> Option<String> {
        if let Some((year, remainder)) = song_date {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_and_total() {
        let (number, total) = parse_number_and_total(None, None).unwrap();
        assert!(number.is_none());
        assert!(total.is_none());

        let (number, total) = parse_number_and_total(Some("10"), None).unwrap();
        assert_eq!(number, Some(10));
        assert!(total.is_none());

        let (number, total) = parse_number_and_total(None, Some("20")).unwrap();
        assert!(number.is_none());
        assert_eq!(total, Some(20));

        let (number, total) = parse_number_and_total(Some("10"), Some("20")).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));

        let (number, total) = parse_number_and_total(Some("10/20"), None).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));

        let (number, total) = parse_number_and_total(Some("10/20"), Some("30")).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));
    }
}
