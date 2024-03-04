use super::common::{extract_common_tags, parse_number_and_total, SongTag};
use crate::OSError;

use anyhow::Result;
use itertools::Itertools;
use lofty::id3::v2::{FrameId, Id3v2Tag, Id3v2Version};
use std::borrow::Cow;

pub const V4_MULTI_VALUE_SEPARATOR: char = '\0';

pub const ARTIST_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPE1"));
pub const ALBUM_ARTIST_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPE2"));
pub const TRACK_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TRCK"));
pub const DISC_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPOS"));

fn extract_texts(
    tag: &mut Id3v2Tag,
    frame_id: &FrameId<'_>,
    multi_value_separator: char,
) -> Option<Vec<String>> {
    tag.get_text(frame_id).map(|text| {
        match tag.original_version() {
            Id3v2Version::V4 => text.split(V4_MULTI_VALUE_SEPARATOR),
            _ => text.split(multi_value_separator),
        }
        .map(str::to_string)
        .collect_vec()
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

        let artists = extract_texts(tag, &ARTIST_ID, multi_value_separator)
            .ok_or_else(|| OSError::NotFound("Artist".into()))?;
        let album_artists = extract_texts(tag, &ALBUM_ARTIST_ID, multi_value_separator);

        let (track_number, track_total) = extract_number_and_total(tag, &TRACK_ID)?;
        let (disc_number, disc_total) = extract_number_and_total(tag, &DISC_ID)?;

        Ok(Self {
            title,
            album,
            artists,
            album_artists,
            track_number,
            track_total,
            disc_number,
            disc_total,
        })
    }
}

#[cfg(test)]
pub mod test {
    pub use super::ALBUM_ARTIST_ID;
    pub use super::ARTIST_ID;
    pub use super::DISC_ID;
    pub use super::TRACK_ID;
    pub use super::V4_MULTI_VALUE_SEPARATOR;
}
