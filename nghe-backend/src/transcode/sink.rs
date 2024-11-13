use std::ffi::CStr;
use std::fmt::Debug;
use std::io::Write;

use derivative::Derivative;
use fs4::fs_std::FileExt;
use loole::{Receiver, Sender};
use nghe_api::common::format;
use rsmpeg::avformat::{AVIOContextContainer, AVIOContextCustom};
use rsmpeg::avutil::AVMem;
use rsmpeg::ffi;
use tracing::instrument;
use typed_path::Utf8NativePath;

use crate::{config, Error};

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Sink {
    #[derivative(Debug = "ignore")]
    tx: Sender<Vec<u8>>,
    buffer_size: usize,
    format: format::Transcode,
    file: Option<std::fs::File>,
}

impl Sink {
    pub async fn new(
        config: &config::Transcode,
        format: format::Transcode,
        output: Option<impl AsRef<Utf8NativePath> + Debug + Send + 'static>,
    ) -> Result<(Self, Receiver<Vec<u8>>), Error> {
        let (tx, rx) = crate::sync::channel(config.channel_size);
        // It will fail in two cases:
        //  - The file already exists because of `create_new`.
        //  - The lock can not be acquired. In this case, another process is already writing to this
        //    file.
        // In both cases, we could start transcoding without writing to a file.
        let span = tracing::Span::current();
        let file = tokio::task::spawn_blocking(move || {
            let _entered = span.enter();
            output.map(Self::lock_write).transpose().ok().flatten()
        })
        .await?;
        Ok((Self { tx, buffer_size: config.buffer_size, format, file }, rx))
    }

    pub fn format(&self) -> &'static CStr {
        // TODO: Use ffmpeg format code after https://github.com/larksuite/rsmpeg/pull/196
        match self.format {
            format::Transcode::Aac => c"output.aac",
            format::Transcode::Flac => c"output.flac",
            format::Transcode::Mp3 => c"output.mp3",
            format::Transcode::Opus => c"output.opus",
            format::Transcode::Wav => c"output.wav",
            format::Transcode::Wma => c"output.wma",
        }
    }

    #[instrument(err(level = "debug"))]
    pub fn lock_write(path: impl AsRef<Utf8NativePath> + Debug) -> Result<std::fs::File, Error> {
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_ref())?;
        file.try_lock_exclusive()?;
        Ok(file)
    }

    #[instrument(err(level = "debug"))]
    pub fn lock_read(path: impl AsRef<Utf8NativePath> + Debug) -> Result<std::fs::File, Error> {
        let file = if cfg!(windows) {
            // On Windows, the file must be open with write permissions to lock it.
            std::fs::OpenOptions::new().read(true).write(true).open(path.as_ref())?
        } else {
            std::fs::OpenOptions::new().read(true).open(path.as_ref())?
        };
        file.try_lock_shared()?;
        Ok(file)
    }

    fn write(&mut self, data: &[u8]) -> i32 {
        let write_len = data.len().try_into().unwrap_or(ffi::AVERROR_BUG2);

        let send_result = self.tx.send(data.to_vec());
        let write_result = self.file.as_mut().map(|file| file.write_all(data));

        tracing::trace!(?write_len, ?send_result, ?write_result);

        // We will keep continue writing in one of two cases below:
        //  - We can still send data to the receiver. We don't care if we can write or not
        //    (including the case where the file is none).
        //  - We can write to the file (this means the file must not be none).
        if send_result.is_ok() || write_result.is_some_and(|result| result.is_ok()) {
            write_len
        } else {
            ffi::AVERROR_OUTPUT_CHANGED
        }
    }
}

impl From<Sink> for AVIOContextContainer {
    fn from(mut sink: Sink) -> Self {
        AVIOContextContainer::Custom(AVIOContextCustom::alloc_context(
            AVMem::new(sink.buffer_size),
            true,
            Vec::default(),
            None,
            Some(Box::new(move |_, data| sink.write(data))),
            None,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::transcode::Status;

    impl Sink {
        pub fn status(&self, status: Status) -> Status {
            match status {
                Status::WithCache if self.file.is_none() => Status::NoCache,
                _ => status,
            }
        }
    }
}
