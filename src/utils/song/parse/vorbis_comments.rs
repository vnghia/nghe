use std::str::FromStr;

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::ogg::{OggPictureStorage, VorbisComments};
use lofty::Picture;

use super::common::{extract_common_tags, parse_track_and_disc};
use super::tag::{SongDate, SongTag};
use crate::config::parsing::VorbisCommentsParsingConfig;

impl SongTag {
    pub fn from_vorbis_comments(
        tag: &mut VorbisComments,
        parsing_config: &VorbisCommentsParsingConfig,
    ) -> Result<Self> {
        let (title, album) = extract_common_tags(tag)?;

        let artists = tag.remove(&parsing_config.artist).collect_vec();
        let album_artists = tag.remove(&parsing_config.album_artist).collect_vec();

        let ((track_number, track_total), (disc_number, disc_total)) = parse_track_and_disc(
            tag.get(&parsing_config.track_number),
            tag.get(&parsing_config.track_total),
            tag.get(&parsing_config.disc_number),
            tag.get(&parsing_config.disc_total),
        )?;

        let date = SongDate::parse(tag.get(&parsing_config.date))?;
        let release_date = SongDate::parse(tag.get(&parsing_config.release_date))?;
        let original_release_date =
            SongDate::parse(tag.get(&parsing_config.original_release_date))?;

        let languages =
            tag.remove(&parsing_config.language).map(|s| Language::from_str(&s)).try_collect()?;

        let picture = Self::extract_ogg_picture(tag);

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
            picture,
        })
    }

    pub fn extract_ogg_picture<O: OggPictureStorage>(o: &mut O) -> Option<Picture> {
        if !o.pictures().is_empty() { Some(o.remove_picture(0).0) } else { None }
    }
}

#[cfg(test)]
mod test {
    use fake::{Fake, Faker};
    use lofty::Accessor;

    use super::*;

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
                tag.push(parsing_config.language.to_owned(), language.to_639_3().to_owned())
            });

            if let Some(picture) = self.picture {
                tag.insert_picture(picture, None).unwrap();
            }

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
