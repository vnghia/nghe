use crate::{models::*, OSResult, OpenSubsonicError};

use concat_string::concat_string;
use itertools::Itertools;
use lofty::{AudioFile, FileType, ItemKey, ParseOptions, ParsingMode, Probe, TaggedFileExt};
use std::io::Cursor;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SongTag {
    pub title: String,
    pub duration: i32,
    pub album: String,
    #[cfg_attr(test, dummy(faker = "(fake::Faker, 2..4)"))]
    pub artists: Vec<String>,
    #[cfg_attr(test, dummy(faker = "(fake::Faker, 1..2)"))]
    pub album_artists: Vec<String>,
}

impl SongTag {
    pub fn parse<B: AsRef<[u8]>, P: AsRef<Path>>(data: B, song_path: P) -> OSResult<SongTag> {
        let song_path_str = song_path
            .as_ref()
            .to_str()
            .expect("non utf-8 path encountered");

        let file_type =
            FileType::from_path(song_path.as_ref()).ok_or(OpenSubsonicError::BadRequest {
                message: Some(
                    concat_string!(song_path_str, " does not have a valid supported extension")
                        .into(),
                ),
            })?;

        let mut tagged_file = Probe::new(Cursor::new(data))
            .options(ParseOptions::new().parsing_mode(ParsingMode::Strict))
            .set_file_type(file_type)
            .read()?;

        let properties = tagged_file.properties();

        let duration = properties.duration().as_secs_f32().ceil() as i32;

        let tag = tagged_file
            .primary_tag_mut()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(
                    concat_string!(song_path_str, " does not have the correct tag type").into(),
                ),
            })?;

        let title = tag
            .take(&ItemKey::TrackTitle)
            .next()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(concat_string!(song_path_str, " title tag not found").into()),
            })?
            .into_value()
            .into_string()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(concat_string!(song_path_str, " title tag is not string").into()),
            })?;

        let album = tag
            .take(&ItemKey::AlbumTitle)
            .next()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(concat_string!(song_path_str, " album tag not found").into()),
            })?
            .into_value()
            .into_string()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(concat_string!(song_path_str, " album tag is not string").into()),
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
            duration,
            album,
            artists,
            album_artists,
        })
    }

    pub fn to_new_or_update_song<'a, S: AsRef<str> + 'a>(
        &'a self,
        music_folder_id: Uuid,
        album_id: Uuid,
        song_file_hash: u64,
        song_file_size: u64,
        song_relative_path: Option<&'a S>,
    ) -> songs::NewOrUpdateSong<'a> {
        songs::NewOrUpdateSong {
            title: (&self.title).into(),
            duration: self.duration,
            album_id,
            music_folder_id,
            path: song_relative_path.map(|path| path.as_ref().into()),
            file_hash: song_file_hash as i64,
            file_size: song_file_size as i64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{
        song::file_type::{to_extension, SONG_FILE_TYPES},
        test::{
            asset::{get_media_asset_duration, get_media_asset_path},
            fs::TemporaryFs,
        },
    };

    use fake::{Fake, Faker};
    use std::fs::read;

    #[test]
    fn test_parse_media_file() {
        for file_type in SONG_FILE_TYPES {
            let path = get_media_asset_path(&file_type);
            let data = read(&path).unwrap();
            let tag = SongTag::parse(data, &path).unwrap();
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
            assert_eq!(tag.duration, get_media_asset_duration(&file_type))
        }
    }

    #[test]
    fn test_parse_media_file_default_value() {
        let fs = TemporaryFs::new();
        let path = fs
            .create_random_paths(1, 1, &[to_extension(&FileType::Flac)])
            .remove(0);
        fs.create_media_file(
            &path,
            SongTag {
                album_artists: Vec::default(),
                ..Faker.fake()
            },
        );
        let new_song_tag = SongTag::parse(read(&path).unwrap(), &path).unwrap();

        assert_eq!(
            new_song_tag.album_artists.iter().sorted().collect_vec(),
            new_song_tag.artists.iter().sorted().collect_vec()
        );
    }
}
