use super::common::{extract_common_tags, parse_track_and_disc};
use super::tag::{SongDate, SongTag};
use crate::config::parsing::Id3v2ParsingConfig;
use crate::OSError;

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::id3::v2::{FrameId, Id3v2Tag, Id3v2Version};
use std::str::{FromStr, Split};

const V4_MULTI_VALUE_SEPARATOR: char = '\0';

fn extract_and_split_str<'a>(
    tag: &'a mut Id3v2Tag,
    frame_id: &FrameId<'_>,
    multi_value_separator: char,
) -> Option<Split<'a, char>> {
    tag.get_text(frame_id)
        .map(|text| match tag.original_version() {
            Id3v2Version::V4 => text.split(V4_MULTI_VALUE_SEPARATOR),
            _ => text.split(multi_value_separator),
        })
}

impl SongTag {
    pub fn from_id3v2(tag: &mut Id3v2Tag, parsing_config: &Id3v2ParsingConfig) -> Result<Self> {
        let (title, album) = extract_common_tags(tag)?;

        let artists = extract_and_split_str(tag, &parsing_config.artist, parsing_config.separator)
            .map(|v| v.map(String::from).collect_vec())
            .ok_or_else(|| OSError::NotFound("Artist".into()))?;
        let album_artists =
            extract_and_split_str(tag, &parsing_config.album_artist, parsing_config.separator)
                .map_or_else(Vec::default, |v| v.map(String::from).collect_vec());

        let ((track_number, track_total), (disc_number, disc_total)) = parse_track_and_disc(
            tag.get_text(&parsing_config.track_number),
            None,
            tag.get_text(&parsing_config.disc_number),
            None,
        )?;

        let date = SongDate::parse(tag.get_text(&parsing_config.date))?;
        let release_date = SongDate::parse(tag.get_text(&parsing_config.release_date))?;
        let original_release_date =
            SongDate::parse(tag.get_text(&parsing_config.original_release_date))?;

        let languages =
            extract_and_split_str(tag, &parsing_config.language, parsing_config.separator)
                .map_or_else(
                    || Ok(Vec::default()),
                    |v| v.map(Language::from_str).try_collect(),
                )?;

        Ok(Self {
            title,
            album,
            artists,
            album_artists,
            track_number,
            track_total,
            disc_number,
            disc_total,
            date,
            release_date,
            original_release_date,
            languages,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use concat_string::concat_string;
    use fake::{Fake, Faker};
    use lofty::{
        id3::v2::{Frame, FrameFlags, TextInformationFrame},
        Accessor,
    };

    fn write_id3v2_text_tag(tag: &mut Id3v2Tag, frame_id: FrameId<'static>, value: String) {
        tag.insert(
            Frame::new(
                frame_id,
                TextInformationFrame {
                    encoding: lofty::TextEncoding::UTF8,
                    value,
                },
                FrameFlags::default(),
            )
            .unwrap(),
        );
    }

    fn write_number_and_total_tag(
        tag: &mut Id3v2Tag,
        frame_id: FrameId<'static>,
        number: Option<u32>,
        total: Option<u32>,
    ) {
        if number.is_some() || total.is_some() {
            write_id3v2_text_tag(
                tag,
                frame_id,
                concat_string!(
                    number.map_or("-1".to_owned(), |i| i.to_string()),
                    total.map_or_else(String::default, |i| concat_string!("/", i.to_string()))
                ),
            );
        }
    }

    impl SongTag {
        pub fn into_id3v2(self, parsing_config: &Id3v2ParsingConfig) -> Id3v2Tag {
            let parsing_config = parsing_config.clone();
            let multi_value_separator = V4_MULTI_VALUE_SEPARATOR.to_string();

            let mut tag = Id3v2Tag::new();
            tag.set_title(self.title);
            tag.set_album(self.album);

            if !self.artists.is_empty() {
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.artist.clone(),
                    self.artists.join(&multi_value_separator),
                );
            }
            if !self.album_artists.is_empty() {
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.album_artist,
                    self.album_artists.join(&multi_value_separator),
                );
            }

            write_number_and_total_tag(
                &mut tag,
                parsing_config.track_number,
                self.track_number,
                self.track_total,
            );
            write_number_and_total_tag(
                &mut tag,
                parsing_config.disc_number,
                self.disc_number,
                self.disc_total,
            );

            if let Some(date) = self.date.to_string() {
                write_id3v2_text_tag(&mut tag, parsing_config.date, date);
            }
            if let Some(date) = self.release_date.to_string() {
                write_id3v2_text_tag(&mut tag, parsing_config.release_date, date);
            }
            if let Some(date) = self.original_release_date.to_string() {
                write_id3v2_text_tag(&mut tag, parsing_config.original_release_date, date);
            }

            if !self.languages.is_empty() {
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.language,
                    self.languages
                        .iter()
                        .map(Language::to_639_3)
                        .join(&multi_value_separator),
                );
            }

            tag
        }
    }

    #[test]
    fn test_round_trip() {
        let config = Id3v2ParsingConfig::default();
        let song_tag: SongTag = Faker.fake();
        assert_eq!(
            song_tag,
            SongTag::from_id3v2(&mut song_tag.clone().into_id3v2(&config), &config).unwrap()
        );
    }
}
