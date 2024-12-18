use lofty::file::AudioFile;
use lofty::id3::v2::Id3v2Tag;
use lofty::mpeg::MpegFile;

use super::tag::id3v2::Has;
use super::{Metadata, Property};
use crate::file::audio::{self, Album, Artists, Genres, NameDateMbz, TrackDisc};
use crate::file::picture::Picture;
use crate::{Error, config, error};

impl<'a> Has<'a> for MpegFile {
    fn tag(&'a self) -> Result<&'a Id3v2Tag, Error> {
        self.id3v2().ok_or_else(|| error::Kind::MissingId3V2Tag(audio::Format::Mpeg).into())
    }
}

impl<'a> Metadata<'a> for MpegFile {
    fn song(&'a self, config: &'a config::Parsing) -> Result<NameDateMbz<'a>, Error> {
        self.tag()?.song(config)
    }

    fn album(&'a self, config: &'a config::Parsing) -> Result<Album<'a>, Error> {
        self.tag()?.album(config)
    }

    fn artists(&'a self, config: &'a config::Parsing) -> Result<Artists<'a>, Error> {
        self.tag()?.artists(config)
    }

    fn track_disc(&'a self, config: &'a config::Parsing) -> Result<TrackDisc, Error> {
        self.tag()?.track_disc(config)
    }

    fn languages(&'a self, config: &'a config::Parsing) -> Result<Vec<isolang::Language>, Error> {
        self.tag()?.languages(config)
    }

    fn genres(&'a self, config: &'a config::Parsing) -> Result<Genres<'a>, Error> {
        Metadata::genres(self.tag()?, config)
    }

    fn picture(&'a self) -> Result<Option<Picture<'static, 'a>>, Error> {
        self.tag()?.picture()
    }
}

impl Property for MpegFile {
    fn property(&self) -> Result<audio::Property, Error> {
        let properties = self.properties();
        Ok(audio::Property {
            duration: properties.duration().try_into()?,
            bitrate: properties.audio_bitrate(),
            bit_depth: None,
            sample_rate: properties.sample_rate(),
            channel_count: properties.channels(),
        })
    }
}
