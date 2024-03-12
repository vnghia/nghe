use crate::config::parsing::VorbisCommentsParsingConfig;

use super::common::{extract_common_tags, parse_number_and_total};
use super::tag::{SongDate, SongTag};

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::ogg::VorbisComments;
use std::str::FromStr;

fn extract_number_and_total(
    tag: &mut VorbisComments,
    number_keys: &str,
    total_keys: &str,
) -> Result<(Option<u32>, Option<u32>)> {
    parse_number_and_total(tag.get(number_keys), tag.get(total_keys))
}

impl SongTag {
    pub fn from_vorbis_comments(
        tag: &mut VorbisComments,
        parsing_config: &VorbisCommentsParsingConfig,
    ) -> Result<Self> {
        let (title, album) = extract_common_tags(tag)?;

        let artists = tag.remove(&parsing_config.artist).collect_vec();
        let album_artists = tag.remove(&parsing_config.album_artist).collect_vec();

        let (track_number, track_total) = extract_number_and_total(
            tag,
            &parsing_config.track_number,
            &parsing_config.track_total,
        )?;
        let (disc_number, disc_total) =
            extract_number_and_total(tag, &parsing_config.disc_number, &parsing_config.disc_total)?;

        let date = SongDate::parse(tag.get(&parsing_config.date))?;
        let release_date = SongDate::parse(tag.get(&parsing_config.release_date))?;
        let original_release_date =
            SongDate::parse(tag.get(&parsing_config.original_release_date))?;

        let languages = tag
            .remove(&parsing_config.language)
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
    use fake::{Fake, Faker};
    use lofty::Accessor;

    impl SongTag {
        pub fn into_vorbis_comments(
            self,
            parsing_config: &VorbisCommentsParsingConfig,
        ) -> VorbisComments {
            let parsing_config = parsing_config.clone();

            let mut tag = VorbisComments::new();
            tag.set_title(self.title);
            tag.set_album(self.album);

            self.artists
                .into_iter()
                .for_each(|artist| tag.push(parsing_config.artist.to_owned(), artist));
            self.album_artists
                .into_iter()
                .for_each(|artist| tag.push(parsing_config.album_artist.to_owned(), artist));

            if let Some(track_number) = self.track_number {
                tag.push(parsing_config.track_number, track_number.to_string());
            }
            if let Some(track_total) = self.track_total {
                tag.push(parsing_config.track_total, track_total.to_string());
            }
            if let Some(disc_number) = self.disc_number {
                tag.push(parsing_config.disc_number, disc_number.to_string());
            }
            if let Some(disc_total) = self.disc_total {
                tag.push(parsing_config.disc_total, disc_total.to_string());
            }

            if let Some(date) = self.date.to_string() {
                tag.push(parsing_config.date, date)
            }
            if let Some(date) = self.release_date.to_string() {
                tag.push(parsing_config.release_date, date)
            }
            if let Some(date) = self.original_release_date.to_string() {
                tag.push(parsing_config.original_release_date, date)
            }

            self.languages.into_iter().for_each(|language| {
                tag.push(
                    parsing_config.language.to_owned(),
                    language.to_639_3().to_owned(),
                )
            });

            tag
        }
    }

    #[test]
    fn test_round_trip() {
        let config = VorbisCommentsParsingConfig::default();
        let song_tag: SongTag = Faker.fake();
        assert_eq!(
            song_tag,
            SongTag::from_vorbis_comments(
                &mut song_tag.clone().into_vorbis_comments(&config),
                &config
            )
            .unwrap()
        );
    }
}
