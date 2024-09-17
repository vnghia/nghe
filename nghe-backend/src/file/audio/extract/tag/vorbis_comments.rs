use std::borrow::Cow;
use std::str::FromStr;

use color_eyre::eyre::OptionExt;
use isolang::Language;
use itertools::Itertools;
use lofty::ogg::VorbisComments;
use uuid::Uuid;

use crate::file::audio::{extract, Artist, Artists, Date, NameDateMbz, TrackDisc};
use crate::{config, Error};

impl Date {
    fn extract_vorbis_comments(tag: &VorbisComments, key: &Option<String>) -> Result<Self, Error> {
        if let Some(key) = key {
            tag.get(key).map(Date::from_str).transpose().map(Option::unwrap_or_default)
        } else {
            Ok(Self::default())
        }
    }
}

impl<'a> NameDateMbz<'a> {
    fn extract_vorbis_comments(
        tag: &'a VorbisComments,
        config: &'a config::parsing::vorbis_comments::Common,
    ) -> Result<Self, Error> {
        Ok(Self {
            name: tag.get(&config.name).ok_or_eyre("Could not extract name")?.into(),
            date: Date::extract_vorbis_comments(tag, &config.date)?,
            release_date: Date::extract_vorbis_comments(tag, &config.release_date)?,
            original_release_date: Date::extract_vorbis_comments(
                tag,
                &config.original_release_date,
            )?,
            mbz_id: tag.get(&config.mbz_id).map(Uuid::from_str).transpose()?,
        })
    }
}

impl<'a> Artist<'a> {
    fn extract_vorbis_comments(
        tag: &'a VorbisComments,
        config: &'a config::parsing::vorbis_comments::Artist,
    ) -> Result<Vec<Self>, Error> {
        let names = tag.get_all(&config.name);
        let mbz_ids = tag.get_all(&config.mbz_id).map(Uuid::from_str);
        let artists = names
            .zip_longest(mbz_ids)
            .map(|iter| match iter {
                itertools::EitherOrBoth::Both(name, mbz_id) => Ok(Self {
                    name: name.into(),
                    mbz_id: mbz_id
                        .map(|mbz_id| if mbz_id.is_nil() { None } else { Some(mbz_id) })?,
                }),
                itertools::EitherOrBoth::Left(name) => Ok(Self { name: name.into(), mbz_id: None }),
                itertools::EitherOrBoth::Right(_) => Err(Error::MediaArtistMbzIdMoreThanArtistName),
            })
            .try_collect()?;
        Ok(artists)
    }
}

impl<'a> extract::Metadata<'a> for VorbisComments {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        NameDateMbz::extract_vorbis_comments(self, &config.vorbis_comments.song)
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        NameDateMbz::extract_vorbis_comments(self, &config.vorbis_comments.album)
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        Artists::new(
            Artist::extract_vorbis_comments(self, &config.vorbis_comments.artists.song)?,
            Artist::extract_vorbis_comments(self, &config.vorbis_comments.artists.album)?,
            self.get(&config.vorbis_comments.compilation).is_some_and(|s| !s.is_empty()),
        )
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        let config::parsing::vorbis_comments::TrackDisc {
            track_number,
            track_total,
            disc_number,
            disc_total,
        } = &config.vorbis_comments.track_disc;
        TrackDisc::parse(
            self.get(track_number),
            self.get(track_total),
            self.get(disc_number),
            self.get(disc_total),
        )
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<isolang::Language>, Error> {
        Ok(self.get_all(&config.vorbis_comments.languages).map(Language::from_str).try_collect()?)
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Vec<Cow<'a, str>>, Error> {
        Ok(self.get_all(&config.vorbis_comments.genres).map(Cow::Borrowed).collect())
    }
}
