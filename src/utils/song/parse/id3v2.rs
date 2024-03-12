use super::common::{extract_common_tags, parse_number_and_total};
use super::tag::SongTag;
use crate::{utils::time::parse_date, OSError};

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::id3::v2::{FrameId, Id3v2Tag, Id3v2Version};
use std::{
    borrow::Cow,
    str::{FromStr, Split},
};

const V4_MULTI_VALUE_SEPARATOR: char = '\0';

const ARTIST_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPE1"));
const ALBUM_ARTIST_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPE2"));
const TRACK_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TRCK"));
const DISC_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TPOS"));

const RECORDING_TIME_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TDRC"));
const RELEASE_TIME_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TDRL"));
const ORIGINAL_RELEASE_TIME_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TDOR"));

const LANGUAGE_ID: FrameId<'static> = FrameId::Valid(Cow::Borrowed("TLAN"));

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

fn extract_number_and_total(
    tag: &mut Id3v2Tag,
    number_id: &FrameId<'_>,
) -> Result<(Option<u32>, Option<u32>)> {
    parse_number_and_total(tag.get_text(number_id), None)
}

impl SongTag {
    pub fn from_id3v2(tag: &mut Id3v2Tag, multi_value_separator: char) -> Result<Self> {
        let (title, album) = extract_common_tags(tag)?;

        let artists = extract_and_split_str(tag, &ARTIST_ID, multi_value_separator)
            .map(|v| v.map(String::from).collect_vec())
            .ok_or_else(|| OSError::NotFound("Artist".into()))?;
        let album_artists = extract_and_split_str(tag, &ALBUM_ARTIST_ID, multi_value_separator)
            .map_or_else(Vec::default, |v| v.map(String::from).collect_vec());

        let (track_number, track_total) = extract_number_and_total(tag, &TRACK_ID)?;
        let (disc_number, disc_total) = extract_number_and_total(tag, &DISC_ID)?;

        let date = parse_date(tag.get_text(&RECORDING_TIME_ID))?;
        let release_date = parse_date(tag.get_text(&RELEASE_TIME_ID))?;
        let original_release_date = parse_date(tag.get_text(&ORIGINAL_RELEASE_TIME_ID))?;

        let languages = extract_and_split_str(tag, &LANGUAGE_ID, multi_value_separator)
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
    use crate::utils::song::test::song_date_to_string;

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
        pub fn into_id3v2(self) -> Id3v2Tag {
            let multi_value_separator = V4_MULTI_VALUE_SEPARATOR.to_string();

            let mut tag = Id3v2Tag::new();
            tag.set_title(self.title);
            tag.set_album(self.album);

            if !self.artists.is_empty() {
                tag.set_artist(self.artists.join(&multi_value_separator));
            }
            if !self.album_artists.is_empty() {
                write_id3v2_text_tag(
                    &mut tag,
                    ALBUM_ARTIST_ID,
                    self.album_artists.join(&multi_value_separator),
                );
            }

            write_number_and_total_tag(&mut tag, TRACK_ID, self.track_number, self.track_total);
            write_number_and_total_tag(&mut tag, DISC_ID, self.disc_number, self.disc_total);

            if let Some(date) = song_date_to_string(&self.date) {
                write_id3v2_text_tag(&mut tag, RECORDING_TIME_ID, date);
            }
            if let Some(date) = song_date_to_string(&self.release_date) {
                write_id3v2_text_tag(&mut tag, RELEASE_TIME_ID, date);
            }
            if let Some(date) = song_date_to_string(&self.original_release_date) {
                write_id3v2_text_tag(&mut tag, ORIGINAL_RELEASE_TIME_ID, date);
            }

            if !self.languages.is_empty() {
                write_id3v2_text_tag(
                    &mut tag,
                    LANGUAGE_ID,
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
        let song_tag: SongTag = Faker.fake();
        assert_eq!(
            song_tag,
            SongTag::from_vorbis_comments(&mut song_tag.clone().into_vorbis_comments()).unwrap()
        );
    }
}
