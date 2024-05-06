use std::str::FromStr;

use anyhow::Result;
use isolang::Language;
use itertools::Itertools;
use lofty::ogg::{OggPictureStorage, VorbisComments};
use lofty::picture::Picture;
use uuid::Uuid;

use super::common::{parse_track_and_disc, to_artist_no_ids};
use super::tag::{MediaDateMbz, SongDate, SongTag};
use crate::config::parsing::{
    MediaDateMbzVorbisCommentsParsingConfig, VorbisCommentsParsingConfig,
};
use crate::models::*;
use crate::OSError;

fn extract_date(tag: &VorbisComments, key: &Option<String>) -> Result<SongDate> {
    if let Some(ref key) = key { SongDate::parse(tag.get(key)) } else { Ok(SongDate(None)) }
}

impl SongTag {
    pub fn from_vorbis_comments(
        tag: &mut VorbisComments,
        parsing_config: &VorbisCommentsParsingConfig,
    ) -> Result<Self> {
        let song = MediaDateMbz::from_vorbis_comments(tag, &parsing_config.song)?;
        let album = MediaDateMbz::from_vorbis_comments(tag, &parsing_config.album)?;

        let artist_names = tag.remove(&parsing_config.artist).collect_vec();
        let artist_mbz_ids = tag.get_all(&parsing_config.artist_mbz_id).collect_vec();
        let artists = to_artist_no_ids(artist_names, Some(artist_mbz_ids))?;

        let album_artist_names = tag.remove(&parsing_config.album_artist).collect_vec();
        let album_artist_mbz_ids = tag.get_all(&parsing_config.album_artist_mbz_id).collect_vec();
        let album_artists = to_artist_no_ids(album_artist_names, Some(album_artist_mbz_ids))?;

        let ((track_number, track_total), (disc_number, disc_total)) = parse_track_and_disc(
            tag.get(&parsing_config.track_number),
            tag.get(&parsing_config.track_total),
            tag.get(&parsing_config.disc_number),
            tag.get(&parsing_config.disc_total),
        )?;

        let languages =
            tag.remove(&parsing_config.language).map(|s| Language::from_str(&s)).try_collect()?;
        let genres = tag.remove(&parsing_config.genre).map(genres::Genre::from).collect();
        let compilation = tag.get(&parsing_config.compilation).is_some_and(|s| !s.is_empty());

        let picture = Self::extract_ogg_picture(tag);

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
            compilation,
            picture,
        })
    }

    pub fn extract_ogg_picture<O: OggPictureStorage>(o: &mut O) -> Option<Picture> {
        if !o.pictures().is_empty() { Some(o.remove_picture(0).0) } else { None }
    }
}

impl MediaDateMbz {
    fn from_vorbis_comments(
        tag: &VorbisComments,
        parsing_config: &MediaDateMbzVorbisCommentsParsingConfig,
    ) -> Result<Self> {
        Ok(Self {
            name: tag
                .get(&parsing_config.name)
                .ok_or_else(|| OSError::NotFound(parsing_config.name.to_owned().into()))?
                .to_owned(),
            date: extract_date(tag, &parsing_config.date)?,
            release_date: extract_date(tag, &parsing_config.release_date)?,
            original_release_date: extract_date(tag, &parsing_config.original_release_date)?,
            mbz_id: tag.get(&parsing_config.mbz_id).map(Uuid::parse_str).transpose()?,
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    impl SongTag {
        pub fn into_vorbis_comments(
            self,
            parsing_config: &VorbisCommentsParsingConfig,
        ) -> VorbisComments {
            let parsing_config = parsing_config.clone();

            let mut tag = VorbisComments::new();

            self.song.into_vorbis_comments(&mut tag, parsing_config.song.clone());
            self.album.into_vorbis_comments(&mut tag, parsing_config.album.clone());

            self.artists.into_iter().for_each(|v| {
                let (name, mbz_id) = v.into();
                tag.push(parsing_config.artist.to_owned(), name);
                tag.push(parsing_config.artist_mbz_id.to_owned(), mbz_id);
            });
            self.album_artists.into_iter().for_each(|v| {
                let (name, mbz_id) = v.into();
                tag.push(parsing_config.album_artist.to_owned(), name);
                tag.push(parsing_config.album_artist_mbz_id.to_owned(), mbz_id);
            });

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

            self.languages.into_iter().for_each(|language| {
                tag.push(parsing_config.language.to_owned(), language.to_639_3().to_owned())
            });
            self.genres.into_iter().for_each(|genre| {
                tag.push(parsing_config.genre.to_owned(), genre.value.into_owned())
            });
            if self.compilation {
                tag.push(parsing_config.compilation, "1".into());
            }

            if let Some(picture) = self.picture {
                tag.insert_picture(picture, None).unwrap();
            }

            tag
        }
    }

    impl MediaDateMbz {
        fn into_vorbis_comments(
            self,
            tag: &mut VorbisComments,
            parsing_config: MediaDateMbzVorbisCommentsParsingConfig,
        ) {
            tag.push(parsing_config.name, self.name);
            if let Some(date) = self.date.to_string() {
                tag.push(parsing_config.date.unwrap(), date)
            }
            if let Some(date) = self.release_date.to_string() {
                tag.push(parsing_config.release_date.unwrap(), date)
            }
            if let Some(date) = self.original_release_date.to_string() {
                tag.push(parsing_config.original_release_date.unwrap(), date)
            }
            if let Some(mbz_id) = self.mbz_id {
                tag.push(parsing_config.mbz_id.to_owned(), mbz_id.to_string());
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
