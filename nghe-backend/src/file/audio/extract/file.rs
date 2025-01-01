use lofty::file::AudioFile;
use lofty::flac::FlacFile;
use lofty::id3::v2::Id3v2Tag;
use lofty::mpeg::MpegFile;
use lofty::ogg::VorbisComments;

use super::{Metadata, Property};
use crate::file::audio::{self, Album, Artists, Genres, NameDateMbz, TrackDisc};
use crate::file::lyric::Lyric;
use crate::file::picture::Picture;
use crate::{Error, config, error};

trait Tag<'a> {
    type Tag: Metadata<'a>;
    fn tag(&'a self) -> Result<&'a Self::Tag, Error>;
}

default impl<'a, T: Tag<'a>> Metadata<'a> for T {
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
        self.tag()?.genres(config)
    }

    fn lyrics(&'a self, config: &'a config::Parsing) -> Result<Vec<Lyric<'a>>, Error> {
        self.tag()?.lyrics(config)
    }

    fn picture(&'a self) -> Result<Option<Picture<'a>>, Error> {
        self.tag()?.picture()
    }
}

impl<'a> Tag<'a> for FlacFile {
    type Tag = VorbisComments;
    fn tag(&'a self) -> Result<&'a VorbisComments, Error> {
        self.vorbis_comments()
            .ok_or_else(|| error::Kind::MissingVorbisComments(audio::Format::Flac).into())
    }
}

impl<'a> Metadata<'a> for FlacFile {
    fn picture(&'a self) -> Result<Option<Picture<'a>>, Error> {
        Picture::extrat_ogg_picture_storage(self)
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

impl<'a> Tag<'a> for MpegFile {
    type Tag = Id3v2Tag;
    fn tag(&'a self) -> Result<&'a Id3v2Tag, Error> {
        self.id3v2().ok_or_else(|| error::Kind::MissingId3V2Tag(audio::Format::Mpeg).into())
    }
}

impl Metadata<'_> for MpegFile {}

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
