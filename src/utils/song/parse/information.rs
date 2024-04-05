use std::io::{Read, Seek};

use anyhow::Result;
use itertools::Itertools;
use lofty::flac::FlacFile;
use lofty::mpeg::MpegFile;
use lofty::{AudioFile, FileProperties, FileType, ParseOptions, ParsingMode};
use tracing::instrument;
use uuid::Uuid;

use super::property::SongProperty;
use super::tag::SongTag;
use crate::config::parsing::ParsingConfig;
use crate::models::*;
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

        let (song_tag, file_property): (_, FileProperties) = match file_type {
            FileType::Flac => {
                let mut flac_file = FlacFile::read_from(reader, parse_options)?;
                let song_tag = SongTag::from_vorbis_comments(
                    flac_file.vorbis_comments_mut().ok_or_else(|| {
                        OSError::NotFound("Vorbis comments inside flac file".into())
                    })?,
                    &parsing_config.vorbis,
                )?;
                // Pictures in flac file are stored directly in that file and not its tag.
                let song_tag = if song_tag.picture.is_none()
                    && let Some(picture) = SongTag::extract_ogg_picture(&mut flac_file)
                {
                    SongTag { picture: Some(picture), ..song_tag }
                } else {
                    song_tag
                };
                (song_tag, (*flac_file.properties()).into())
            }
            FileType::Mpeg => {
                let mut mp3_file = MpegFile::read_from(reader, parse_options)?;
                let id3v2_tag = mp3_file
                    .id3v2_mut()
                    .ok_or_else(|| OSError::NotFound("Id3v2 inside mp3 file".into()))?;
                let song_tag = SongTag::from_id3v2(id3v2_tag, &parsing_config.id3v2)?;
                (song_tag, (*mp3_file.properties()).into())
            }
            _ => unreachable!("not supported file type: {:?}", file_type),
        };

        if song_tag.artists.is_empty() {
            anyhow::bail!(OSError::NotFound("Artist".into()));
        }

        let song_property = SongProperty {
            format: file_type,
            duration: file_property.duration().as_secs_f32(),
            bitrate: file_property
                .audio_bitrate()
                .ok_or_else(|| OSError::NotFound("Audio bitrate".into()))?,
            sample_rate: file_property
                .sample_rate()
                .ok_or_else(|| OSError::NotFound("Sample rate".into()))?,
            channel_count: file_property
                .channels()
                .ok_or_else(|| OSError::NotFound("Channel count".into()))?,
        };

        Ok(Self { tag: song_tag, property: song_property })
    }

    pub fn to_update_information_db(
        &self,
        album_id: Uuid,
        file_hash: i64,
        file_size: i64,
        cover_art_id: Option<Uuid>,
    ) -> songs::SongUpdateInformationDB<'_> {
        let (year, month, day) =
            self.tag.song.date_or_default().or(self.tag.album.date_or_default()).to_ymd();
        let (release_year, release_month, release_day) =
            self.tag.song.release_date_or_default().to_ymd();
        let (original_release_year, original_release_month, original_release_day) =
            self.tag.song.original_release_date.to_ymd();

        songs::SongUpdateInformationDB {
            // Song tag
            title: (&self.tag.song.name).into(),
            album_id,
            track_number: self.tag.track_number.map(|i| i as _),
            track_total: self.tag.track_total.map(|i| i as _),
            disc_number: self.tag.disc_number.map(|i| i as _),
            disc_total: self.tag.disc_total.map(|i| i as _),
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
            bitrate: self.property.bitrate as _,
            sample_rate: self.property.sample_rate as _,
            channel_count: self.property.channel_count as _,
            // Filesystem property
            file_hash,
            file_size,
            // Foreign key columns
            cover_art_id,
        }
    }

    pub fn to_full_information_db<'a, S: AsRef<str> + 'a>(
        &'a self,
        album_id: Uuid,
        file_hash: i64,
        file_size: i64,
        cover_art_id: Option<Uuid>,
        music_folder_id: Uuid,
        relative_path: &'a S,
    ) -> songs::SongFullInformationDB<'a> {
        let update_information =
            self.to_update_information_db(album_id, file_hash, file_size, cover_art_id);

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
    use crate::utils::test::asset::{get_asset_dir, get_media_asset_path};

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

            let song = tag.song;
            assert_eq!(song.name, "Sample", "{:?} song name does not match", file_type);
            assert_eq!(song.date.0, None, "{:?} song date does not match", file_type);
            assert_eq!(
                song.release_date.0, None,
                "{:?} song release date does not match",
                file_type
            );
            assert_eq!(
                song.original_release_date.0, None,
                "{:?} song original release date does not match",
                file_type
            );
            assert_eq!(song.mbz_id, None, "{:?} song mbz id does not match", file_type);

            let album = tag.album;
            assert_eq!(album.name, "Album", "{:?} album name does not match", file_type);
            assert_eq!(
                album.date.0,
                Some((2000, Some((12, Some(31))))),
                "{:?} album date does not match",
                file_type
            );
            assert_eq!(
                album.release_date.0, None,
                "{:?} album release date does not match",
                file_type
            );
            assert_eq!(
                album.original_release_date.0,
                Some((3000, Some((1, None)))),
                "{:?} album original release date does not match",
                file_type
            );
            assert_eq!(album.mbz_id, None, "{:?} album mbz id does not match", file_type);

            assert_eq!(
                tag.artists,
                Vec::<artists::ArtistNoId>::from([
                    ("Artist1", Some(uuid::uuid!("1ffedd2d-f63d-4dc2-9332-d3132e5134ac"))).into(),
                    ("Artist2", None).into()
                ]),
                "{:?} artists does not match",
                file_type
            );
            assert_eq!(
                tag.album_artists,
                Vec::<artists::ArtistNoId>::from([
                    ("Artist1", None).into(),
                    ("Artist3", None).into()
                ]),
                "{:?} album artists does not match",
                file_type
            );

            assert_eq!(tag.track_number, Some(10), "{:?} track number does not match", file_type);
            assert_eq!(tag.track_total, None, "{:?} track total does not match", file_type);
            assert_eq!(tag.disc_number, Some(5), "{:?} disc number does not match", file_type);
            assert_eq!(tag.disc_total, Some(10), "{:?} disc total does not match", file_type);

            assert_eq!(
                tag.languages.into_iter().sorted().collect_vec(),
                [Language::Eng, Language::Vie],
                "{:?} language does not match",
                file_type
            );

            let picture_data =
                std::fs::read(get_asset_dir().join("test").join("sample.jpg")).unwrap();
            assert_eq!(
                tag.picture.unwrap().into_data(),
                picture_data,
                "{:?} picture does not match",
                file_type
            );
        }
    }
}
