use std::fs::File;
use std::io::{Result as IoResult, Write};

use atomic_write_file::AtomicWriteFile;
use educe::Educe;
use ffmpeg_next::format as avformat;
use loole::Sender;
use nghe_api::format;
use typed_path::Utf8PlatformPathBuf;

use crate::Error;
#[derive(Debug)]
pub struct Args {
    pub format: format::Transcode,
    pub cache: Option<Utf8PlatformPathBuf>,
}

#[derive(Educe)]
#[educe(Debug)]
pub struct Sink {
    pub format: format::Transcode,
    #[educe(Debug(ignore))]
    pub tx: Sender<Vec<u8>>,
    pub cache: Option<File>,
}

impl Args {
    pub fn open_cache(&self) -> Result<Option<AtomicWriteFile>, Error> {
        self.cache.as_ref().map(AtomicWriteFile::open).transpose().map_err(Into::into)
    }
}

impl Sink {
    pub fn try_into_context(self, buffer_size: usize) -> Result<avformat::context::Output, Error> {
        let filename = self.filename();
        avformat::output_to_stream(
            avformat::context::StreamIo::from_write_with_capacity(self, buffer_size)?,
            Some(filename),
            None,
        )
        .map_err(Into::into)
    }

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
