use super::common::{extract_common_tags, parse_number_and_total};
use super::tag::SongTag;
use crate::utils::time::parse_date;

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::ogg::VorbisComments;
use std::str::FromStr;

const ARTIST_KEY: &str = "ARTIST";
const ALBUM_ARTIST_KEYS: &[&str] = &["ALBUMARTIST", "ALBUM ARTIST"];

const TRACK_NUMBER_KEYS: &[&str] = &["TRACKNUMBER", "TRACKNUM"];
const TRACK_TOTAL_KEYS: &[&str] = &["TRACKTOTAL", "TOTALTRACKS"];
const DISC_NUMBER_KEYS: &[&str] = &["DISCNUMBER"];
const DISC_TOTAL_KEYS: &[&str] = &["DISCTOTAL", "TOTALDISCS"];

const DATE_KEY: &str = "DATE";
const ORIGINAL_RELEASE_DATE_KEYS: &[&str] = &["ORIGYEAR", "ORIGINALDATE"];

const LANGUAGE: &str = "LANGUAGE";

fn extract_number_and_total(
    tag: &mut VorbisComments,
    number_keys: &[&str],
    total_keys: &[&str],
) -> Result<(Option<u32>, Option<u32>)> {
    let number_value = number_keys.iter().find_map(|key| tag.get(key));
    let total_value = total_keys.iter().find_map(|key| tag.get(key));
    parse_number_and_total(number_value, total_value)
}

impl SongTag {
    pub fn from_vorbis_comments(tag: &mut VorbisComments) -> Result<Self> {
        let (title, album) = extract_common_tags(tag)?;

        let artists = tag.remove(ARTIST_KEY).collect_vec();
        let album_artists = ALBUM_ARTIST_KEYS
            .iter()
            .map(|key| tag.remove(key).collect_vec())
            .find(|vec| !vec.is_empty())
            .unwrap_or_default();

        let (track_number, track_total) =
            extract_number_and_total(tag, TRACK_NUMBER_KEYS, TRACK_TOTAL_KEYS)?;
        let (disc_number, disc_total) =
            extract_number_and_total(tag, DISC_NUMBER_KEYS, DISC_TOTAL_KEYS)?;

        let date = parse_date(tag.get(DATE_KEY))?;
        let release_date = None;
        let original_release_date = parse_date(
            ORIGINAL_RELEASE_DATE_KEYS
                .iter()
                .find_map(|key| tag.get(key)),
        )?;

        let languages = tag
            .remove(LANGUAGE)
            .map(|s| Language::from_str(&s))
            .try_collect()?;

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
    use crate::utils::song::test::song_date_to_string;

    use fake::{Fake, Faker};
    use lofty::Accessor;

    impl SongTag {
        pub fn into_vorbis_comments(self) -> VorbisComments {
            let mut tag = VorbisComments::new();
            tag.set_title(self.title);
            tag.set_album(self.album);

            self.artists
                .into_iter()
                .for_each(|artist| tag.push(ARTIST_KEY.to_owned(), artist));
            self.album_artists
                .into_iter()
                .for_each(|artist| tag.push(ALBUM_ARTIST_KEYS[0].to_owned(), artist));

            if let Some(track_number) = self.track_number {
                tag.push(TRACK_NUMBER_KEYS[0].to_owned(), track_number.to_string());
            }
            if let Some(track_total) = self.track_total {
                tag.push(TRACK_TOTAL_KEYS[0].to_owned(), track_total.to_string());
            }
            if let Some(disc_number) = self.disc_number {
                tag.push(DISC_NUMBER_KEYS[0].to_owned(), disc_number.to_string());
            }
            if let Some(disc_total) = self.disc_total {
                tag.push(DISC_TOTAL_KEYS[0].to_owned(), disc_total.to_string());
            }

            if let Some(date) = song_date_to_string(&self.date) {
                tag.push(DATE_KEY.to_owned(), date)
            }
            if let Some(date) = song_date_to_string(&self.original_release_date) {
                tag.push(ORIGINAL_RELEASE_DATE_KEYS[0].to_owned(), date)
            }

            self.languages
                .into_iter()
                .for_each(|language| tag.push(LANGUAGE.to_owned(), language.to_639_3().to_owned()));

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
