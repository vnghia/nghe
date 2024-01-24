use crate::{OSResult, OpenSubsonicError};

use itertools::Itertools;
use lofty::{Accessor, FileType, ItemKey, ParseOptions, ParsingMode, Probe, TaggedFileExt};
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SongTag {
    pub title: String,
    pub album: String,
    pub artists: Vec<String>,
}

impl SongTag {
    pub fn parse<T: AsRef<[u8]>>(data: T, file_type: FileType) -> OSResult<SongTag> {
        let tagged_file = Probe::new(Cursor::new(data))
            .options(ParseOptions::new().parsing_mode(ParsingMode::Strict))
            .set_file_type(file_type)
            .read()?;

        let tag = tagged_file
            .primary_tag()
            .ok_or_else(|| OpenSubsonicError::NotFound {
                message: Some("file does not have the correct tag type".to_owned()),
            })?;

        let title = tag
            .title()
            .ok_or_else(|| OpenSubsonicError::NotFound {
                message: Some("title tag not found".to_owned()),
            })?
            .to_string();
        let album = tag
            .album()
            .ok_or_else(|| OpenSubsonicError::NotFound {
                message: Some("album tag not found".to_owned()),
            })?
            .to_string();
        let artists = tag
            .get_strings(&ItemKey::TrackArtist)
            .map(std::string::ToString::to_string)
            .collect_vec();

        Ok(SongTag {
            title,
            album,
            artists,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{media::file_type::MEDIA_FILE_TYPES, test::asset::get_media_asset_path};

    use std::fs::read;

    #[test]
    fn test_parse_media_file() {
        for file_type in MEDIA_FILE_TYPES {
            let data = read(get_media_asset_path(&file_type)).unwrap();
            let tag = SongTag::parse(data, file_type).unwrap();
            assert_eq!(tag.title, "Sample");
            assert_eq!(tag.album, "Album");
            assert_eq!(tag.artists, ["Artist1", "Artist2"]);
        }
    }
}
