use std::ffi::CStr;
use std::ops::{Deref, DerefMut};

use file_guard::{try_lock, FileGuard};
use loole::{Receiver, Sender};
use nghe_api::common::format;
use rsmpeg::avformat::{AVIOContextContainer, AVIOContextCustom};
use rsmpeg::avutil::AVMem;
use rsmpeg::ffi;
use typed_path::Utf8NativePath;

use crate::{config, Error};

pub struct Lock {
    inner: std::fs::File,
}

impl Deref for Lock {
    type Target = std::fs::File;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Lock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Lock {
    pub fn write(path: impl AsRef<Utf8NativePath>) -> Result<FileGuard<Self>, Error> {
        let inner = std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_ref())?;
        let file = Self { inner };
        Ok(try_lock(file, file_guard::Lock::Exclusive, 0, 0)?)
    }

    pub fn read(path: impl AsRef<Utf8NativePath>) -> Result<FileGuard<Self>, Error> {
        let inner = std::fs::OpenOptions::new().read(true).write(true).open(path.as_ref())?;
        let file = Self { inner };
        Ok(try_lock(file, file_guard::Lock::Shared, 0, 0)?)
    }
}

pub struct Sink {
    tx: Sender<Vec<u8>>,
    buffer_size: usize,
    format: format::Transcode,
    file: Option<FileGuard<Lock>>,
}

impl Sink {
    pub fn new(
        config: &config::Transcode,
        format: format::Transcode,
        output: Option<impl AsRef<Utf8NativePath>>,
    ) -> (Self, Receiver<Vec<u8>>) {
        let (tx, rx) = crate::sync::channel(config.channel_size);
        // It will fail in two cases:
        //  - The file already exists because of `create_new`.
        //  - The lock can not be acquired. In this case, another process is already writing to this
        //    file.
        // In both cases, we could start transcoding without writing to a file.
        let file = output.map(Lock::write).transpose().ok().flatten();
        (Self { tx, buffer_size: config.buffer_size, format, file }, rx)
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

    fn write(&self, data: impl Into<Vec<u8>>) -> i32 {
        let data = data.into();
        let write_len = data.len().try_into().unwrap_or(ffi::AVERROR_BUG2);

        match self.tx.send(data) {
            Ok(()) => write_len,
            Err(_) => ffi::AVERROR_OUTPUT_CHANGED,
        }
    }
}

impl From<Sink> for AVIOContextContainer {
    fn from(sink: Sink) -> Self {
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
