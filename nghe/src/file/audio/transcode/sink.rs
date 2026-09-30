use std::io::{Result as IoResult, Write};

use educe::Educe;
use ffmpeg_next::format::context::StreamIo;
use loole::Sender;
use nghe_api::common::format;

use crate::Error;

#[derive(Educe)]
#[educe(Debug)]
pub struct Sink {
    #[educe(Debug(ignore))]
    pub tx: Sender<Vec<u8>>,
    pub buffer_size: usize,
    pub format: format::Transcode,
    pub cache: Option<std::fs::File>,
}

impl Sink {
    pub fn filename(&self) -> &'static str {
        match self.format {
            format::Transcode::Aac => ".aac",
            format::Transcode::Flac => ".flac",
            format::Transcode::Mp3 => ".mp3",
            format::Transcode::Opus => ".opus",
            format::Transcode::Wav => ".wav",
            format::Transcode::Wma => ".wma",
        }
    }
}

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let write_len = buf.len();

        let send_result = self.tx.send(buf.to_vec());
        let cache_write_result = self.cache.as_mut().map(|file| file.write_all(buf));

        tracing::trace!(?write_len, ?send_result, ?cache_write_result);

        // We will keep continue writing in one of two cases below:
        //  - We can still send data to the receiver. We don't care if we can write or not
        //    (including the case where the cache is none).
        //  - We can write to the cache (this means the cache must not be none).
        if send_result.is_ok() || cache_write_result.is_some_and(|result| result.is_ok()) {
            Ok(write_len)
        } else {
            Err(std::io::Error::other("Could not send data to neither recv nor cache"))
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        self.tx.close();
        self.cache.as_mut().map(Write::flush).transpose()?;
        Ok(())
    }
}

impl TryFrom<Sink> for StreamIo {
    type Error = Error;

    fn try_from(sink: Sink) -> Result<Self, Self::Error> {
        let buffer_size = sink.buffer_size;
        Self::from_write_with_capacity(sink, buffer_size).map_err(Error::from)
    }
}
