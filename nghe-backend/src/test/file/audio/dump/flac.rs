use lofty::flac::FlacFile;
use lofty::ogg::{OggPictureStorage as _, VorbisComments};

use super::Metadata;
use super::tag::vorbis_comments::Has;
use crate::file::picture::Picture;

impl Has for FlacFile {
    fn tag(&mut self) -> &mut VorbisComments {
        self.vorbis_comments_mut().unwrap()
    }
}

impl Metadata for FlacFile {
    fn dump_picture(&mut self, picture: Option<Picture<'_, '_>>) -> &mut Self {
        if let Some(picture) = picture {
            self.insert_picture(picture.into(), None).unwrap();
        }
        self
    }
}
