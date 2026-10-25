use std::io::{Read, Result as IoResult};

use educe::Educe;
use ffmpeg_next::format as avformat;

use crate::{Error, file, filesystem};

#[derive(Educe)]
#[educe(Debug)]
pub struct Args {
    pub format: file::audio::Format,
    #[educe(Debug(ignore))]
    pub source: filesystem::io::Read,
}

pub type Source = Args;

impl Source {
    pub fn try_into_context(self, buffer_size: usize) -> Result<avformat::context::Input, Error> {
        let filename = self.filename();
        avformat::input_from_stream(
            avformat::context::StreamIo::from_read_with_capacity(self, buffer_size)?,
            Some(filename),
            None,
        )
        .map_err(Into::into)
    }

    pub fn filename(&self) -> &'static str {
        match self.format {
            file::audio::Format::Flac => ".flac",
            file::audio::Format::Mpeg => ".mp3",
        }
    }
}

impl Read for Source {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.source.read(buf)
    }
}
