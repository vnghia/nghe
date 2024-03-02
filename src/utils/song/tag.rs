use crate::{models::*, OSResult, OpenSubsonicError};

use concat_string::concat_string;
use derivative::Derivative;
use itertools::Itertools;
use lofty::{AudioFile, FileType, ItemKey, ParseOptions, ParsingMode, Probe, Tag, TaggedFileExt};
use std::io::Cursor;
use std::path::Path;
use uuid::Uuid;

#[derive(Derivative, Debug, Clone)]
#[derivative(PartialEq)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SongTag {
    pub title: String,
    #[derivative(PartialEq = "ignore")]
    pub duration: f32,
    pub album: String,
    #[cfg_attr(test, dummy(faker = "(fake::Faker, 2..4)"))]
    pub artists: Vec<String>,
    #[cfg_attr(test, dummy(faker = "(fake::Faker, 1..2)"))]
    pub album_artists: Vec<String>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
}

fn take_string(tag: &mut Tag, key: &ItemKey) -> Option<String> {
    if let Some(item) = tag.take(key).next() {
        item.into_value().into_string()
    } else {
        None
    }
}

fn take_number_and_total(
    tag: &mut Tag,
    number_key: &ItemKey,
    total_key: &ItemKey,
) -> OSResult<(Option<u32>, Option<u32>)> {
    if let Some(number_value) = take_string(tag, number_key) {
        if let Some((number_value, total_value)) = number_value.split_once('/') {
            Ok((Some(number_value.parse()?), Some(total_value.parse()?)))
        } else {
            Ok((
                Some(number_value.parse()?),
                if let Some(total_value) = take_string(tag, total_key) {
                    Some(total_value.parse()?)
                } else {
                    None
                },
            ))
        }
    } else {
        Ok((
            None,
            if let Some(total_value) = take_string(tag, total_key) {
                Some(total_value.parse()?)
            } else {
                None
            },
        ))
    }
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

        let duration = properties.duration().as_secs_f32();

        let tag = tagged_file
            .primary_tag_mut()
            .ok_or(OpenSubsonicError::NotFound {
                message: Some(
                    concat_string!(song_path_str, " does not have the correct tag type").into(),
                ),
            })?;

        let title = take_string(tag, &ItemKey::TrackTitle).ok_or(OpenSubsonicError::NotFound {
            message: Some(concat_string!(song_path_str, " title tag not found").into()),
        })?;

        let album = take_string(tag, &ItemKey::AlbumTitle).ok_or(OpenSubsonicError::NotFound {
            message: Some(concat_string!(song_path_str, " album tag not found").into()),
        })?;

        let (track_number, track_total) =
            take_number_and_total(tag, &ItemKey::TrackNumber, &ItemKey::TrackTotal)?;
        let (disc_number, disc_total) =
            take_number_and_total(tag, &ItemKey::DiscNumber, &ItemKey::DiscTotal)?;

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
            track_number,
            track_total,
            disc_number,
            disc_total,
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
            track_number: self.track_number.map(|i| i as i32),
            track_total: self.track_total.map(|i| i as i32),
            disc_number: self.disc_number.map(|i| i as i32),
            disc_total: self.track_total.map(|i| i as i32),
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
        test::{asset::get_media_asset_path, fs::TemporaryFs},
    };

    use fake::{Fake, Faker};
    use lofty::TagType;
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
            assert_eq!(
                tag.track_number,
                Some(10),
                "{:?} track number does not match",
                file_type
            );
            assert_eq!(
                tag.track_total, None,
                "{:?} track total does not match",
                file_type
            );
            assert_eq!(
                tag.disc_number,
                Some(5),
                "{:?} disc number does not match",
                file_type
            );
            assert_eq!(
                tag.disc_total,
                Some(10),
                "{:?} disc total does not match",
                file_type
            );
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

    #[test]
    fn test_take_number_and_total_number_only() {
        let mut tag = Tag::new(TagType::VorbisComments);
        tag.insert_text(ItemKey::TrackNumber, "10".to_owned());
        let (number, total) =
            take_number_and_total(&mut tag, &ItemKey::TrackNumber, &ItemKey::TrackTotal).unwrap();
        assert_eq!(number, Some(10));
        assert!(total.is_none());
    }

    #[test]
    fn test_take_number_and_total_total_only() {
        let mut tag = Tag::new(TagType::VorbisComments);
        tag.insert_text(ItemKey::TrackTotal, "20".to_owned());
        let (number, total) =
            take_number_and_total(&mut tag, &ItemKey::TrackNumber, &ItemKey::TrackTotal).unwrap();
        assert!(number.is_none());
        assert_eq!(total, Some(20));
    }

    #[test]
    fn test_take_number_and_total_number() {
        let mut tag = Tag::new(TagType::VorbisComments);
        tag.insert_text(ItemKey::TrackNumber, "10".to_owned());
        tag.insert_text(ItemKey::TrackTotal, "20".to_owned());
        let (number, total) =
            take_number_and_total(&mut tag, &ItemKey::TrackNumber, &ItemKey::TrackTotal).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));
    }

    #[test]
    fn test_take_number_and_total_with_separator() {
        let mut tag = Tag::new(TagType::VorbisComments);
        tag.insert_text(ItemKey::TrackNumber, "10/20".to_owned());
        let (number, total) =
            take_number_and_total(&mut tag, &ItemKey::TrackNumber, &ItemKey::TrackTotal).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));
    }

    #[test]
    fn test_take_number_and_total_take_separator() {
        let mut tag = Tag::new(TagType::VorbisComments);
        tag.insert_text(ItemKey::TrackNumber, "10/20".to_owned());
        tag.insert_text(ItemKey::TrackTotal, "30".to_owned());
        let (number, total) =
            take_number_and_total(&mut tag, &ItemKey::TrackNumber, &ItemKey::TrackTotal).unwrap();
        assert_eq!(number, Some(10));
        assert_eq!(total, Some(20));
    }
}
