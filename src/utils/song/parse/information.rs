use std::io::{Read, Seek};

use anyhow::Result;
use itertools::Itertools;
use lofty::flac::FlacFile;
use lofty::mpeg::MpegFile;
use lofty::{AudioFile, FileType, ParseOptions, ParsingMode};
use tracing::instrument;
use uuid::Uuid;

use super::property::SongProperty;
use super::tag::SongTag;
use crate::config::parsing::ParsingConfig;
use crate::models::songs;
use crate::utils::song::file_type::to_extension;
use crate::OSError;

pub struct SongInformation {
    pub tag: SongTag,
    pub property: SongProperty,
}

impl SongInformation {
    #[instrument(skip_all, err(Debug))]
    pub fn read_from<R: Read + Seek>(
        reader: &mut R,
        file_type: FileType,
        parsing_config: &ParsingConfig,
    ) -> Result<Self> {
        let parse_options = ParseOptions::new().parsing_mode(ParsingMode::Strict);

        let (song_tag, song_property) = match file_type {
            FileType::Flac => {
                let mut flac_file = FlacFile::read_from(reader, parse_options)?;
                let song_tag = SongTag::from_vorbis_comments(
                    flac_file.vorbis_comments_mut().ok_or_else(|| {
                        OSError::NotFound("Vorbis comments inside flac file".into())
                    })?,
                    &parsing_config.vorbis,
                )?;

                let flac_property = flac_file.properties();
                let song_property = SongProperty {
                    format: file_type,
                    duration: flac_property.duration().as_secs_f32(),
                    bitrate: flac_property.audio_bitrate(),
                    sample_rate: flac_property.sample_rate(),
                    channel_count: flac_property.channels(),
                };

                (song_tag, song_property)
            }
            FileType::Mpeg => {
                let mut mp3_file = MpegFile::read_from(reader, parse_options)?;
                let song_tag = SongTag::from_id3v2(
                    mp3_file
                        .id3v2_mut()
                        .ok_or_else(|| OSError::NotFound("Id3v2 inside mp3 file".into()))?,
                    &parsing_config.id3v2,
                )?;

                let mp3_property = mp3_file.properties();
                let song_property = SongProperty {
                    format: file_type,
                    duration: mp3_property.duration().as_secs_f32(),
                    bitrate: mp3_property.audio_bitrate(),
                    sample_rate: mp3_property.sample_rate(),
                    channel_count: mp3_property.channels(),
                };

                (song_tag, song_property)
            }
            _ => unreachable!("not supported file type: {:?}", file_type),
        };

        if song_tag.artists.is_empty() {
            anyhow::bail!(OSError::NotFound("Artist".into()));
        }

        Ok(Self { tag: song_tag, property: song_property })
    }

    pub fn to_update_information_db(
        &self,
        album_id: Uuid,
        file_hash: i64,
        file_size: i64,
    ) -> songs::SongUpdateInformationDB<'_> {
        let (year, month, day) = self.tag.date_or_default().to_ymd();
        let (release_year, release_month, release_day) =
            self.tag.release_date_or_default().to_ymd();
        let (original_release_year, original_release_month, original_release_day) =
            self.tag.original_release_date.to_ymd();

        songs::SongUpdateInformationDB {
            // Song tag
            title: (&self.tag.title).into(),
            album_id,
            track_number: self.tag.track_number.map(|i| i as i32),
            track_total: self.tag.track_total.map(|i| i as i32),
            disc_number: self.tag.disc_number.map(|i| i as i32),
            disc_total: self.tag.disc_total.map(|i| i as i32),
            year,
            month,
            day,
            release_year,
            release_month,
            release_day,
            original_release_year,
            original_release_month,
            original_release_day,
            languages: self.tag.languages.iter().map(|language| language.to_639_3()).collect_vec(),
            // Song property
            format: to_extension(&self.property.format).into(),
            duration: self.property.duration,
            bitrate: self.property.bitrate as i32,
            sample_rate: self.property.sample_rate as i32,
            channel_count: self.property.channel_count as i16,
            // Filesystem property
            file_hash,
            file_size,
        }
    }

    pub fn to_full_information_db<'a, S: AsRef<str> + 'a>(
        &'a self,
        album_id: Uuid,
        file_hash: i64,
        file_size: i64,
        music_folder_id: Uuid,
        relative_path: &'a S,
    ) -> songs::SongFullInformationDB<'a> {
        let update_information = self.to_update_information_db(album_id, file_hash, file_size);

        songs::SongFullInformationDB {
            update_information,
            music_folder_id,
            relative_path: relative_path.as_ref().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use isolang::Language;

    use super::*;
    use crate::utils::song::file_type::SONG_FILE_TYPES;
    use crate::utils::test::asset::get_media_asset_path;

    #[test]
    fn test_parse_media_file() {
        for file_type in SONG_FILE_TYPES {
            let path = get_media_asset_path(&file_type);
            let tag = SongInformation::read_from(
                &mut std::fs::File::open(&path).unwrap(),
                file_type,
                &ParsingConfig::default(),
            )
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
                tag.album_artists.iter().sorted().collect_vec(),
                ["Artist1", "Artist3"],
                "{:?} album artists does not match",
                file_type
            );
            assert_eq!(tag.track_number, Some(10), "{:?} track number does not match", file_type);
            assert_eq!(tag.track_total, None, "{:?} track total does not match", file_type);
            assert_eq!(tag.disc_number, Some(5), "{:?} disc number does not match", file_type);
            assert_eq!(tag.disc_total, Some(10), "{:?} disc total does not match", file_type);
            assert_eq!(
                tag.date.0,
                Some((2000, Some((12, Some(31))))),
                "{:?} date does not match",
                file_type
            );
            assert_eq!(tag.release_date.0, None, "{:?} release date does not match", file_type);
            assert_eq!(
                tag.original_release_date.0,
                Some((3000, Some((1, None)))),
                "{:?} original release date does not match",
                file_type
            );
            assert_eq!(
                tag.languages.into_iter().sorted().collect_vec(),
                [Language::Eng, Language::Vie],
                "{:?} language does not match",
                file_type
            );
        }
    }
}
