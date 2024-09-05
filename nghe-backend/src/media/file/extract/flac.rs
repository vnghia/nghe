use lofty::file::AudioFile;
use lofty::flac::FlacFile;

use super::{MetadataExtractor, Property, PropertyExtractor};
use crate::media::file::{Artists, Common, TrackDisc};
use crate::{config, Error};

impl MetadataExtractor for FlacFile {
    fn song<'a>(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.song(config)
    }

    fn album<'a>(&'a self, config: &'a config::Parsing) -> Result<Common<'a>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.album(config)
    }

    fn artists<'a>(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.artists(config)
    }

    fn track_disc<'a>(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.track_disc(config)
    }

    fn languages<'a>(
        &'a self,
        config: &'a config::Parsing,
    ) -> Result<Vec<isolang::Language>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.languages(config)
    }

    fn genres<'a>(&'a self, config: &'a config::Parsing) -> Result<Vec<&'a str>, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.genres(config)
    }

    fn compilation<'a>(&'a self, config: &'a config::Parsing) -> Result<bool, Error> {
        self.vorbis_comments().ok_or(Error::MediaFlacMissingVorbisComments)?.compilation(config)
    }
}

impl PropertyExtractor for FlacFile {
    fn property(&self) -> Result<Property, Error> {
        let properties = self.properties();
        Ok(Property {
            duration: properties.duration().as_secs_f32(),
            bitrate: properties.audio_bitrate(),
            bit_depth: Some(properties.bit_depth()),
            sample_rate: properties.sample_rate(),
            channel_count: properties.channels(),
        })
    }
}
