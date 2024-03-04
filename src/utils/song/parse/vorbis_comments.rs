use super::common::{extract_common_tags, parse_number_and_total, SongTag};
use crate::OSError;

use anyhow::Result;
use itertools::Itertools;
use lofty::ogg::VorbisComments;

pub const ARTIST_KEY: &str = "ARTIST";
pub const ALBUM_ARTIST_KEYS: &[&str] = &["ALBUMARTIST", "ALBUM ARTIST"];

const TRACK_NUMBER_KEYS: &[&str] = &["TRACKNUMBER", "TRACKNUM"];
const TRACK_TOTAL_KEYS: &[&str] = &["TRACKTOTAL", "TOTALTRACKS"];
const DISC_NUMBER_KEYS: &[&str] = &["DISCNUMBER"];
const DISC_TOTAL_KEYS: &[&str] = &["DISCTOTAL", "TOTALDISCS"];

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

        let artists = {
            let artists = tag.remove(ARTIST_KEY).collect_vec();
            if !artists.is_empty() {
                artists
            } else {
                anyhow::bail!(OSError::NotFound("Artist".into()))
            }
        };
        let album_artists = ALBUM_ARTIST_KEYS
            .iter()
            .map(|key| tag.remove(key).collect_vec())
            .find(|vec| !vec.is_empty());

        let (track_number, track_total) =
            extract_number_and_total(tag, TRACK_NUMBER_KEYS, TRACK_TOTAL_KEYS)?;
        let (disc_number, disc_total) =
            extract_number_and_total(tag, DISC_NUMBER_KEYS, DISC_TOTAL_KEYS)?;

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
    pub use super::ALBUM_ARTIST_KEYS;
    pub use super::ARTIST_KEY;
}
