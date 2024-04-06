use std::borrow::Cow;
use std::str::{FromStr, Split};

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::id3::v2::{FrameId, FrameValue, Id3v2Tag, Id3v2Version};
use lofty::Picture;
use uuid::Uuid;

use super::common::{parse_track_and_disc, to_artist_no_ids};
use super::tag::{MediaDateMbz, SongDate, SongTag};
use crate::config::parsing::{
    FrameIdOrUserText, Id3v2ParsingConfig, MediaDateMbzId3v2ParsingConfig,
};
use crate::models::*;
use crate::OSError;

const V4_MULTI_VALUE_SEPARATOR: char = '\0';
const PICTURE_FRAME_ID: FrameId<'_> = FrameId::Valid(Cow::Borrowed("APIC"));

fn get_text<'a>(tag: &'a Id3v2Tag, key: &FrameIdOrUserText) -> Option<&'a str> {
    match key {
        FrameIdOrUserText::FrameId(frame_id) => tag.get_text(frame_id),
        FrameIdOrUserText::UserText(user_text) => tag.get_user_text(user_text),
    }
}

fn extract_and_split_str<'a>(
    tag: &'a Id3v2Tag,
    key: &FrameIdOrUserText,
    multi_value_separator: char,
) -> Option<Split<'a, char>> {
    get_text(tag, key).map(|text| match tag.original_version() {
        Id3v2Version::V4 => text.split(V4_MULTI_VALUE_SEPARATOR),
        _ => text.split(multi_value_separator),
    })
}

fn extract_date(tag: &Id3v2Tag, key: &Option<FrameIdOrUserText>) -> Result<SongDate> {
    if let Some(ref key) = key { SongDate::parse(get_text(tag, key)) } else { Ok(SongDate(None)) }
}

impl SongTag {
    pub fn from_id3v2(tag: &mut Id3v2Tag, parsing_config: &Id3v2ParsingConfig) -> Result<Self> {
        let song = MediaDateMbz::from_id3v2(tag, &parsing_config.song)?;
        let album = MediaDateMbz::from_id3v2(tag, &parsing_config.album)?;

        let artist_names =
            extract_and_split_str(tag, &parsing_config.artist, parsing_config.separator)
                .map(|v| v.map(String::from).collect_vec())
                .ok_or_else(|| OSError::NotFound("Artist".into()))?;
        let artist_mbz_ids =
            extract_and_split_str(tag, &parsing_config.artist_mbz_id, parsing_config.separator)
                .map(|v| v.collect_vec());
        let artists = to_artist_no_ids(artist_names, artist_mbz_ids)?;

        let album_artist_names =
            extract_and_split_str(tag, &parsing_config.album_artist, parsing_config.separator)
                .map_or_else(Vec::default, |v| v.map(String::from).collect_vec());
        let album_artist_mbz_ids = extract_and_split_str(
            tag,
            &parsing_config.album_artist_mbz_id,
            parsing_config.separator,
        )
        .map(|v| v.collect_vec());
        let album_artists = to_artist_no_ids(album_artist_names, album_artist_mbz_ids)?;

        let ((track_number, track_total), (disc_number, disc_total)) = parse_track_and_disc(
            get_text(tag, &parsing_config.track_number),
            None,
            get_text(tag, &parsing_config.disc_number),
            None,
        )?;

        let languages =
            extract_and_split_str(tag, &parsing_config.language, parsing_config.separator)
                .map_or_else(|| Ok(Vec::default()), |v| v.map(Language::from_str).try_collect())?;
        let genres = extract_and_split_str(tag, &parsing_config.genre, parsing_config.separator)
            .map_or_else(Vec::default, |v| v.map(genres::Genre::from).collect());

        let picture = Self::extract_id3v2_picture(tag)?;

        Ok(Self {
            song,
            album,
            artists,
            album_artists,
            track_number,
            track_total,
            disc_number,
            disc_total,
            languages,
            genres,
            picture,
        })
    }

    pub fn extract_id3v2_picture(tag: &mut Id3v2Tag) -> Result<Option<Picture>> {
        tag.remove(&PICTURE_FRAME_ID)
            .next()
            .map(|f| {
                if let FrameValue::Picture(p) = f.content() {
                    Ok(p.picture.clone())
                } else {
                    Err(OSError::InvalidParameter("Picture frame in id3v2".into()))
                }
            })
            .transpose()
            .map_err(anyhow::Error::from)
    }
}

impl MediaDateMbz {
    fn from_id3v2(tag: &Id3v2Tag, parsing_config: &MediaDateMbzId3v2ParsingConfig) -> Result<Self> {
        Ok(Self {
            name: get_text(tag, &parsing_config.name)
                .ok_or_else(|| OSError::NotFound(parsing_config.name.as_ref().to_owned().into()))?
                .to_owned(),
            date: extract_date(tag, &parsing_config.date)?,
            release_date: extract_date(tag, &parsing_config.release_date)?,
            original_release_date: extract_date(tag, &parsing_config.original_release_date)?,
            mbz_id: get_text(tag, &parsing_config.mbz_id).map(Uuid::parse_str).transpose()?,
        })
    }
}

#[cfg(test)]
mod test {
    use concat_string::concat_string;
    use lofty::id3::v2::{Frame, FrameFlags, TextInformationFrame};

    use super::*;

    fn write_id3v2_text_tag(tag: &mut Id3v2Tag, key: FrameIdOrUserText, value: String) {
        match key {
            FrameIdOrUserText::FrameId(frame_id) => tag.insert(
                Frame::new(
                    frame_id,
                    TextInformationFrame { encoding: lofty::TextEncoding::UTF8, value },
                    FrameFlags::default(),
                )
                .unwrap(),
            ),
            FrameIdOrUserText::UserText(user_text) => tag.insert_user_text(user_text, value),
        };
    }

    fn write_number_and_total_tag(
        tag: &mut Id3v2Tag,
        key: FrameIdOrUserText,
        number: Option<u32>,
        total: Option<u32>,
    ) {
        if number.is_some() || total.is_some() {
            write_id3v2_text_tag(
                tag,
                key,
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

            self.song.into_id3v2(&mut tag, parsing_config.song.clone());
            self.album.into_id3v2(&mut tag, parsing_config.album.clone());

            if !self.artists.is_empty() {
                let (artist_names, artist_mbz_ids): (Vec<String>, Vec<String>) =
                    self.artists.into_iter().map(|v| v.into()).unzip();
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.artist.clone(),
                    artist_names.join(&multi_value_separator),
                );
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.artist_mbz_id.clone(),
                    artist_mbz_ids.join(&multi_value_separator),
                );
            }
            if !self.album_artists.is_empty() {
                let (album_artist_names, album_artist_mbz_ids): (Vec<String>, Vec<String>) =
                    self.album_artists.into_iter().map(|v| v.into()).unzip();
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.album_artist.clone(),
                    album_artist_names.join(&multi_value_separator),
                );
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.album_artist_mbz_id.clone(),
                    album_artist_mbz_ids.join(&multi_value_separator),
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

            if !self.languages.is_empty() {
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.language,
                    self.languages.iter().map(Language::to_639_3).join(&multi_value_separator),
                );
            }
            if !self.genres.is_empty() {
                write_id3v2_text_tag(
                    &mut tag,
                    parsing_config.genre,
                    self.genres.iter().map(|g| g.value.as_ref()).join(&multi_value_separator),
                );
            }

            if let Some(picture) = self.picture {
                tag.insert_picture(picture);
            }

            tag
        }
    }

    impl MediaDateMbz {
        fn into_id3v2(self, tag: &mut Id3v2Tag, parsing_config: MediaDateMbzId3v2ParsingConfig) {
            write_id3v2_text_tag(tag, parsing_config.name, self.name);
            if let Some(date) = self.date.to_string() {
                write_id3v2_text_tag(tag, parsing_config.date.unwrap(), date);
            }
            if let Some(date) = self.release_date.to_string() {
                write_id3v2_text_tag(tag, parsing_config.release_date.unwrap(), date);
            }
            if let Some(date) = self.original_release_date.to_string() {
                write_id3v2_text_tag(tag, parsing_config.original_release_date.unwrap(), date);
            }
            if let Some(mbz_id) = self.mbz_id {
                write_id3v2_text_tag(tag, parsing_config.mbz_id.clone(), mbz_id.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;

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
