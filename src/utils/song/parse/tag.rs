use isolang::Language;

pub type SongDate = Option<(u16, Option<(u8, Option<u8>)>)>;

#[cfg(test)]
pub use fake::{Dummy, Fake, Opt, Optional};

#[cfg(test)]
pub use itertools::Itertools;

#[derive(Debug)]
#[cfg_attr(test, derive(Dummy, Clone, PartialEq, Eq))]
pub struct SongTag {
    pub title: String,
    pub album: String,
    #[cfg_attr(
        test,
        dummy(expr = "fake::vec![String; 1..=5].into_iter().sorted().collect()")
    )]
    pub artists: Vec<String>,
    #[cfg_attr(
        test,
        dummy(expr = "fake::vec![String; 0..=5].into_iter().sorted().collect()")
    )]
    pub album_artists: Vec<String>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    #[cfg_attr(
        test,
        dummy(
            expr = "Opt((0..=9999, Opt((1..=12, Opt(1..=28, 50)), 50)), 50).fake::<Optional<(u16, Optional<(u8, Optional<u8>)>)>>().0.map(|y| (y.0, y.1.0.map(|m| (m.0, m.1.0))))"
        )
    )]
    pub date: SongDate,
    #[cfg_attr(test, dummy(expr = "None"))]
    pub release_date: SongDate,
    #[cfg_attr(
        test,
        dummy(
            expr = "Opt((0..=9999, Opt((1..=12, Opt(1..=28, 50)), 50)), 50).fake::<Optional<(u16, Optional<(u8, Optional<u8>)>)>>().0.map(|y| (y.0, y.1.0.map(|m| (m.0, m.1.0))))"
        )
    )]
    pub original_release_date: SongDate,
    #[cfg_attr(
        test,
        dummy(
            expr = "((0..=7915), 0..=2).fake::<Vec<usize>>().into_iter().map(|i| Language::from_usize(i).unwrap()).sorted().collect()"
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

pub fn song_date_to_ymd(song_date: SongDate) -> (Option<i16>, Option<i16>, Option<i16>) {
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
    use super::*;
    use crate::utils::song::SongInformation;

    impl SongTag {
        pub fn to_information(&self) -> SongInformation {
            SongInformation {
                tag: self.clone(),
                property: Default::default(),
            }
        }
    }

    pub fn song_date_to_string(song_date: &SongDate) -> Option<String> {
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

    pub fn ymd_to_song_date(year: Option<i16>, month: Option<i16>, day: Option<i16>) -> SongDate {
        if let Some(year) = year {
            let year = year as u16;
            if let Some(month) = month {
                let month = month as u8;
                if let Some(day) = day {
                    let day = day as u8;
                    Some((year, Some((month, Some(day)))))
                } else {
                    Some((year, Some((month, None))))
                }
            } else {
                Some((year, None))
            }
        } else {
            None
        }
    }
}
