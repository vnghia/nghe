use lofty::file::AudioFile;
use lofty::flac::FlacFile;
use lofty::ogg::OggPictureStorage as _;

use super::{Metadata, Property};
use crate::file::audio::{self, Album, Artists, Genres, NameDateMbz, TrackDisc};
use crate::file::picture::Picture;
use crate::{Error, config, error};

// TODO: Reduce duplication while getting tag
impl<'a> Metadata<'a> for FlacFile {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        self.vorbis_comments()
            .ok_or(error::Kind::MissingMediaTag("vorbis comments", audio::Format::Flac))?
            .song(config)
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error> {
        self.vorbis_comments()
            .ok_or(error::Kind::MissingMediaTag("vorbis comments", audio::Format::Flac))?
            .album(config)
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        self.vorbis_comments()
            .ok_or(error::Kind::MissingMediaTag("vorbis comments", audio::Format::Flac))?
            .artists(config)
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        self.vorbis_comments()
            .ok_or(error::Kind::MissingMediaTag("vorbis comments", audio::Format::Flac))?
            .track_disc(config)
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<isolang::Language>, Error> {
        self.vorbis_comments()
            .ok_or(error::Kind::MissingMediaTag("vorbis comments", audio::Format::Flac))?
            .languages(config)
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error> {
        self.vorbis_comments()
            .ok_or(error::Kind::MissingMediaTag("vorbis comments", audio::Format::Flac))?
            .genres(config)
    }

    fn picture(&'a self) -> Result<Option<Picture<'static, 'a>>, Error> {
        let mut iter = self.pictures().iter();
        if cfg!(test) {
            iter.find_map(|(picture, _)| {
                if picture
                    .description()
                    .is_some_and(|description| description == Picture::TEST_DESCRIPTION)
                {
                    Some(picture.try_into())
                } else {
                    None
                }
            })
            .transpose()
        } else {
            iter.next().map(|(picture, _)| picture.try_into()).transpose()
        }
    }
}

impl Property for FlacFile {
    fn property(&self) -> Result<audio::Property, Error> {
        let properties = self.properties();
        Ok(audio::Property {
            duration: properties.duration().try_into()?,
            bitrate: properties.audio_bitrate(),
            bit_depth: Some(properties.bit_depth()),
            sample_rate: properties.sample_rate(),
            channel_count: properties.channels(),
        })
    }
}
