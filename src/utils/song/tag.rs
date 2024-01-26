use crate::{models::*, OSResult, OpenSubsonicError};

use itertools::Itertools;
use lofty::{Accessor, FileType, ItemKey, ParseOptions, ParsingMode, Probe, TaggedFileExt};
use std::io::Cursor;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SongTag {
    pub title: String,
    pub album: String,
    #[cfg_attr(test, dummy(faker = "(fake::Faker, 2..4)"))]
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

    pub fn into_new_or_update_song<'a, T: AsRef<str> + 'a>(
        self: SongTag,
        music_folder_id: Uuid,
        album_id: Uuid,
        song_file_hash: u64,
        song_file_size: u64,
        song_relative_path: Option<&'a T>,
    ) -> songs::NewOrUpdateSong<'a> {
        songs::NewOrUpdateSong {
            title: self.title.into(),
            album_id,
            music_folder_id,
            path: song_relative_path.map(|path| std::borrow::Cow::Borrowed(path.as_ref())),
            file_hash: song_file_hash as i64,
            file_size: song_file_size as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{song::file_type::SONG_FILE_TYPES, test::asset::get_media_asset_path};

    use std::fs::read;

    #[test]
    fn test_parse_media_file() {
        for file_type in SONG_FILE_TYPES {
            let data = read(get_media_asset_path(&file_type)).unwrap();
            let tag = SongTag::parse(data, file_type).unwrap();
            assert_eq!(tag.title, "Sample", "{:?} title does not match", file_type);
            assert_eq!(tag.album, "Album", "{:?} album does not match", file_type);
            assert_eq!(
                tag.artists,
                ["Artist1", "Artist2"],
                "{:?} artists does not match",
                file_type
            );
        }
    }
}
