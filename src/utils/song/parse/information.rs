use super::common::{SongProperty, SongTag};
use crate::{models::songs, OSError};

use anyhow::Result;
use lofty::{flac::FlacFile, mpeg::MpegFile, AudioFile, FileType, ParseOptions, ParsingMode};
use std::io::{Read, Seek};
use uuid::Uuid;

pub struct SongInformation {
    pub tag: SongTag,
    pub property: SongProperty,
}

impl SongInformation {
    pub fn read_from<R: Read + Seek>(reader: &mut R, file_type: &FileType) -> Result<Self> {
        let parse_options = ParseOptions::new().parsing_mode(ParsingMode::Strict);

        let (song_tag, song_property) = match file_type {
            FileType::Flac => {
                let mut flac_file = FlacFile::read_from(reader, parse_options)?;
                let song_tag =
                    SongTag::from_vorbis_comments(flac_file.vorbis_comments_mut().ok_or_else(
                        || OSError::NotFound("Vorbis comments inside flac file".into()),
                    )?)?;
                let song_property = SongProperty {
                    duration: flac_file.properties().duration().as_secs_f32(),
                };
                (song_tag, song_property)
            }
            FileType::Mpeg => {
                let mut mp3_file = MpegFile::read_from(reader, parse_options)?;
                let song_tag = SongTag::from_id3v2(
                    mp3_file
                        .id3v2_mut()
                        .ok_or_else(|| OSError::NotFound("Id3v2 inside mp3 file".into()))?,
                    '/',
                )?;
                let song_property = SongProperty {
                    duration: mp3_file.properties().duration().as_secs_f32(),
                };
                (song_tag, song_property)
            }
            _ => unreachable!("not supported file type: {:?}", file_type),
        };

        if song_tag
            .album_artists
            .as_ref()
            .is_some_and(|v| v.is_empty())
        {
            unreachable!("album artists is some but empty")
        }

        Ok(Self {
            tag: song_tag,
            property: song_property,
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
            title: (&self.tag.title).into(),
            duration: self.property.duration,
            album_id,
            track_number: self.tag.track_number.map(|i| i as i32),
            track_total: self.tag.track_total.map(|i| i as i32),
            disc_number: self.tag.disc_number.map(|i| i as i32),
            disc_total: self.tag.track_total.map(|i| i as i32),
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
    use crate::utils::{song::file_type::SONG_FILE_TYPES, test::asset::get_media_asset_path};

    use itertools::Itertools;

    #[test]
    fn test_parse_media_file() {
        for file_type in SONG_FILE_TYPES {
            let path = get_media_asset_path(&file_type);
            let tag =
                SongInformation::read_from(&mut std::fs::File::open(&path).unwrap(), &file_type)
                    .unwrap()
                    .tag;
            assert_eq!(tag.title, "Sample", "{:?} title does not match", file_type);
            assert_eq!(tag.album, "Album", "{:?} album does not match", file_type);
            assert_eq!(
                tag.artists.iter().sorted().collect_vec(),
                ["Artist1", "Artist2"],
                "{:?} artists does not match",
                file_type
            );
            assert_eq!(
                tag.album_artists.unwrap().iter().sorted().collect_vec(),
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
}
