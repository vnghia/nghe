use super::common::{extract_common_tags, parse_number_and_total, SongTag};
use crate::{utils::time::parse_date, OSError};

use anyhow::Result;
use itertools::Itertools;
use lofty::id3::v2::{FrameId, Id3v2Tag, Id3v2Version};
use std::{borrow::Cow, str::Split};

pub const V4_MULTI_VALUE_SEPARATOR: char = '\0';

pub const ARTIST_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPE1"));
pub const ALBUM_ARTIST_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPE2"));
pub const TRACK_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TRCK"));
pub const DISC_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPOS"));

pub const RECORDING_TIME_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TDRC"));
pub const RELEASE_TIME_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TDRL"));
pub const ORIGINAL_RELEASE_TIME_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TDOR"));

fn extract_and_split_str<'a>(
    tag: &'a mut Id3v2Tag,
    frame_id: &FrameId<'_>,
    multi_value_separator: char,
) -> Option<Split<'a, char>> {
    tag.get_text(frame_id)
        .map(|text| match tag.original_version() {
            Id3v2Version::V4 => text.split(V4_MULTI_VALUE_SEPARATOR),
            _ => text.split(multi_value_separator),
        })
}

fn extract_number_and_total(
    tag: &mut Id3v2Tag,
    number_id: &FrameId<'_>,
) -> Result<(Option<u32>, Option<u32>)> {
    parse_number_and_total(tag.get_text(number_id), None)
}

impl SongTag {
    pub fn from_id3v2(tag: &mut Id3v2Tag, multi_value_separator: char) -> Result<Self> {
        let (title, album) = extract_common_tags(tag)?;

        let artists = extract_and_split_str(tag, &ARTIST_ID, multi_value_separator)
            .map(|v| v.map(String::from).collect_vec())
            .ok_or_else(|| OSError::NotFound("Artist".into()))?;
        let album_artists = extract_and_split_str(tag, &ALBUM_ARTIST_ID, multi_value_separator)
            .map_or_else(Vec::default, |v| v.map(String::from).collect_vec());

        let (track_number, track_total) = extract_number_and_total(tag, &TRACK_ID)?;
        let (disc_number, disc_total) = extract_number_and_total(tag, &DISC_ID)?;

        let date = parse_date(tag.get_text(&RECORDING_TIME_ID))?;
        let release_date = parse_date(tag.get_text(&RELEASE_TIME_ID))?;
        let original_release_date = parse_date(tag.get_text(&ORIGINAL_RELEASE_TIME_ID))?;

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
        })
    }
}

#[cfg(test)]
pub mod test {
    pub use super::V4_MULTI_VALUE_SEPARATOR;

    pub use super::ALBUM_ARTIST_ID;
    pub use super::ARTIST_ID;
    pub use super::DISC_ID;
    pub use super::TRACK_ID;

    pub use super::ORIGINAL_RELEASE_TIME_ID;
    pub use super::RECORDING_TIME_ID;
    pub use super::RELEASE_TIME_ID;
}
