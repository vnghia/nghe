use super::common::{extract_common_tags, parse_number_and_total, SongTag};
use crate::utils::time::parse_date;

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::ogg::VorbisComments;
use std::str::FromStr;

pub const ARTIST_KEY: &str = "ARTIST";
pub const ALBUM_ARTIST_KEYS: &[&str] = &["ALBUMARTIST", "ALBUM ARTIST"];

const TRACK_NUMBER_KEYS: &[&str] = &["TRACKNUMBER", "TRACKNUM"];
const TRACK_TOTAL_KEYS: &[&str] = &["TRACKTOTAL", "TOTALTRACKS"];
const DISC_NUMBER_KEYS: &[&str] = &["DISCNUMBER"];
const DISC_TOTAL_KEYS: &[&str] = &["DISCTOTAL", "TOTALDISCS"];

pub const DATE_KEY: &str = "DATE";
pub const ORIGINAL_RELEASE_DATE_KEYS: &[&str] = &["ORIGYEAR", "ORIGINALDATE"];

pub const LANGUAGE: &str = "LANGUAGE";

fn extract_number_and_total(
    tag: &mut VorbisComments,
    number_keys: &[&str],
    total_keys: &[&str],
) -> Result<(Option<u32>, Option<u32>)> {
    let number_value = number_keys.iter().find_map(|key| tag.get(key));
    let total_value = total_keys.iter().find_map(|key| tag.get(key));
    parse_number_and_total(number_value, total_value)
}

impl SongTag {
    pub fn from_vorbis_comments(tag: &mut VorbisComments) -> Result<Self> {
        let (title, album) = extract_common_tags(tag)?;

        let artists = tag.remove(ARTIST_KEY).collect_vec();
        let album_artists = ALBUM_ARTIST_KEYS
            .iter()
            .map(|key| tag.remove(key).collect_vec())
            .find(|vec| !vec.is_empty())
            .unwrap_or_default();

        let (track_number, track_total) =
            extract_number_and_total(tag, TRACK_NUMBER_KEYS, TRACK_TOTAL_KEYS)?;
        let (disc_number, disc_total) =
            extract_number_and_total(tag, DISC_NUMBER_KEYS, DISC_TOTAL_KEYS)?;

        let date = parse_date(tag.get(DATE_KEY))?;
        let release_date = None;
        let original_release_date = parse_date(
            ORIGINAL_RELEASE_DATE_KEYS
                .iter()
                .find_map(|key| tag.get(key)),
        )?;

        let languages = tag
            .remove(LANGUAGE)
            .map(|s| Language::from_str(&s))
            .try_collect()?;

        Ok(Self {
            title,
            album,
            artists,
            album_artists,
            track_number,
            track_total,
            disc_number,
            disc_total,
            date,
            release_date,
            original_release_date,
            languages,
        })
    }
}

#[cfg(test)]
pub mod test {
    pub use super::ALBUM_ARTIST_KEYS;
    pub use super::ARTIST_KEY;

    pub use super::DATE_KEY;
    pub use super::ORIGINAL_RELEASE_DATE_KEYS;

    pub use super::LANGUAGE;
}
