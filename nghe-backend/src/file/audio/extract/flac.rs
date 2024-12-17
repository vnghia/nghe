use lofty::file::AudioFile;
use lofty::flac::FlacFile;
use lofty::ogg::{OggPictureStorage as _, VorbisComments};

use super::tag::vorbis_comments::Has;
use super::{Metadata, Property};
use crate::file::audio;
use crate::file::picture::Picture;
use crate::{Error, error};

impl<'a> Has<'a> for FlacFile {
    fn tag(&'a self) -> Result<&'a VorbisComments, Error> {
        self.vorbis_comments()
            .ok_or_else(|| error::Kind::MissingVorbisComments(audio::Format::Flac).into())
    }
}

impl<'a> Metadata<'a> for FlacFile {
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
