use crate::{models::*, OSResult, OpenSubsonicError};

use itertools::Itertools;
use lofty::{FileType, ItemKey, ParseOptions, ParsingMode, Probe, TaggedFileExt};
use std::borrow::Cow;
use std::io::Cursor;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SongTag {
    pub title: String,
    pub album: String,
    #[cfg_attr(test, dummy(faker = "(fake::Faker, 2..4)"))]
    pub artists: Vec<String>,
    #[cfg_attr(test, dummy(faker = "(fake::Faker, 1..2)"))]
    pub album_artists: Vec<String>,
}

impl SongTag {
    pub fn parse<B: AsRef<[u8]>>(data: B, file_type: FileType) -> OSResult<SongTag> {
        let mut tagged_file = Probe::new(Cursor::new(data))
            .options(ParseOptions::new().parsing_mode(ParsingMode::Strict))
            .set_file_type(file_type)
            .read()?;

        let tag = tagged_file
            .primary_tag_mut()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(Cow::Borrowed("file does not have the correct tag type")),
            })?;

        let title = tag
            .take(&ItemKey::TrackTitle)
            .next()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(Cow::Borrowed("title tag not found")),
            })?
            .into_value()
            .into_string()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(Cow::Borrowed("title tag is not string")),
            })?;

        let album = tag
            .take(&ItemKey::AlbumTitle)
            .next()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(Cow::Borrowed("album tag not found")),
            })?
            .into_value()
            .into_string()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(Cow::Borrowed("album tag is not string")),
            })?;

        let artists = tag.take_strings(&ItemKey::TrackArtist).collect_vec();

        let album_artists = {
            let album_artists = tag.take_strings(&ItemKey::AlbumArtist).collect_vec();
            if album_artists.is_empty() {
                artists.clone()
            } else {
                album_artists
            }
        };

        Ok(SongTag {
            title,
            album,
            artists,
            album_artists,
        })
    }

    pub fn to_new_or_update_song<'a, S: AsRef<str> + 'a>(
        self: &'a SongTag,
        music_folder_id: Uuid,
        album_id: Uuid,
        song_file_hash: u64,
        song_file_size: u64,
        song_relative_path: Option<&'a S>,
    ) -> songs::NewOrUpdateSong<'a> {
        songs::NewOrUpdateSong {
            title: std::borrow::Cow::Borrowed(&self.title),
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
    use fake::{Fake, Faker};

    use super::*;
    use crate::utils::{
        song::file_type::{to_extension, SONG_FILE_TYPES},
        test::{asset::get_media_asset_path, fs::TemporaryFs},
    };

    use std::fs::read;

    #[test]
    fn test_parse_media_file() {
        for file_type in SONG_FILE_TYPES {
            let data = read(get_media_asset_path(&file_type)).unwrap();
            let tag = SongTag::parse(data, file_type).unwrap();
            assert_eq!(tag.title, "Sample", "{:?} title does not match", file_type);
            assert_eq!(tag.album, "Album", "{:?} album does not match", file_type);
            assert_eq!(
                tag.artists.iter().sorted().collect_vec(),
                ["Artist1", "Artist2"],
                "{:?} artists does not match",
                file_type
            );
            assert_eq!(
                tag.album_artists.iter().sorted().collect_vec(),
                ["Artist1", "Artist3"],
                "{:?} album artists does not match",
                file_type
            );
        }
    }

    #[test]
    fn test_parse_media_file_default_value() {
        let fs = TemporaryFs::new();
        let (path, file_type) = fs
            .create_random_paths(1, 1, &[to_extension(&FileType::Flac)])
            .remove(0);
        fs.create_media_file(
            &path,
            SongTag {
                album_artists: Vec::default(),
                ..Faker.fake()
            },
        );
        let new_song_tag = SongTag::parse(read(path).unwrap(), file_type.unwrap()).unwrap();

        assert_eq!(
            new_song_tag.album_artists.iter().sorted().collect_vec(),
            new_song_tag.artists.iter().sorted().collect_vec()
        );
    }
}
