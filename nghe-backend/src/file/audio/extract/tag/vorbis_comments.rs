use std::str::FromStr;

use indexmap::IndexSet;
use isolang::Language;
use itertools::Itertools;
use lofty::ogg::{OggPictureStorage, VorbisComments};
use uuid::Uuid;

use crate::file::audio::{Album, Artist, Artists, Date, Genres, NameDateMbz, TrackDisc, extract};
use crate::file::image::Picture;
use crate::file::lyric::Lyric;
use crate::{Error, config, error};

impl Date {
    fn extract_vorbis_comments(tag: &VorbisComments, key: Option<&str>) -> Result<Self, Error> {
        if let Some(key) = key {
            tag.get(key).map(Self::from_str).transpose().map(Option::unwrap_or_default)
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
            name: tag.get(&config.name).ok_or_else(|| error::Kind::MissingMediaName)?.into(),
            date: Date::extract_vorbis_comments(tag, config.date.as_deref())?,
            release_date: Date::extract_vorbis_comments(tag, config.release_date.as_deref())?,
            original_release_date: Date::extract_vorbis_comments(
                tag,
                config.original_release_date.as_deref(),
            )?,
            mbz_id: tag
                .get(&config.mbz_id)
                .map(|mbz_id| {
                    Uuid::from_str(mbz_id)
                        .map_err(|_| error::Kind::InvalidMbzIdTagFormat(mbz_id.to_owned()))
                })
                .transpose()?,
        })
    }
}

impl<'a> Artist<'a> {
    fn extract_vorbis_comments(
        tag: &'a VorbisComments,
        config: &'a config::parsing::vorbis_comments::Artist,
    ) -> Result<IndexSet<Self>, Error> {
        let names = tag.get_all(&config.name);
        let mbz_ids = tag.get_all(&config.mbz_id);
        Self::try_collect(names, mbz_ids)
    }
}

impl<'a> Picture<'a> {
    pub fn extrat_ogg_picture_storage(
        tag: &'a impl OggPictureStorage,
    ) -> Result<Option<Self>, Error> {
        let mut iter = tag.pictures().iter();
        iter.find_map(|(picture, _)| {
            if !cfg!(test)
                || picture
                    .description()
                    .is_some_and(|description| description == Picture::TEST_DESCRIPTION)
            {
                Some(picture.try_into())
            } else {
                None
            }
        })
        .transpose()
    }
}

impl<'a> extract::Metadata<'a> for VorbisComments {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        NameDateMbz::extract_vorbis_comments(self, &config.vorbis_comments.song)
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error> {
        Album::extract_vorbis_comments(self, &config.vorbis_comments.album)
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
        Ok(self
            .get_all(&config.vorbis_comments.languages)
            .map(|language| Language::from_str(language).map_err(error::Kind::from))
            .try_collect()?)
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error> {
        Ok(self.get_all(&config.vorbis_comments.genres).collect())
    }

    fn lyrics(&'a self, config: &'a config::Parsing) -> Result<Vec<Lyric<'a>>, Error> {
        self.get(&config.vorbis_comments.lyric.unsync)
            .map(|content| Ok(Lyric::from_unsync_text(content)))
            .into_iter()
            .chain(self.get_all(&config.vorbis_comments.lyric.sync).map(Lyric::from_sync_text))
            .try_collect()
    }

    fn picture(&'a self) -> Result<Option<Picture<'a>>, Error> {
        Picture::extrat_ogg_picture_storage(self)
    }
}
