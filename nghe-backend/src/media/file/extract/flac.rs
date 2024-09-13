use std::borrow::Cow;

use lofty::file::AudioFile;
use lofty::flac::FlacFile;

use super::{Metadata, Property};
use crate::media::file::{self, Artists, NameDateMbz, TrackDisc};
use crate::{config, Error};

impl<'a> Metadata<'a> for FlacFile {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.song(config)
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.album(config)
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.artists(config)
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.track_disc(config)
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<isolang::Language>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.languages(config)
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Vec<Cow<'a, str>>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.genres(config)
    }

    fn compilation(&'a self, config: &'a config::Parsing) -> Result<bool, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.compilation(config)
    }
}

impl Property for FlacFile {
    fn property(&self) -> Result<file::Property, Error> {
        let properties = self.properties();
        Ok(file::Property {
            duration: properties.duration().as_secs_f32(),
            bitrate: properties.audio_bitrate(),
            bit_depth: Some(properties.bit_depth()),
            sample_rate: properties.sample_rate(),
            channel_count: properties.channels(),
        })
    }
}