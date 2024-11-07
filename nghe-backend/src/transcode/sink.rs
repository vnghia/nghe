use std::ffi::CStr;

use nghe_api::common::format;
use rsmpeg::avformat::{AVIOContextContainer, AVIOContextCustom};
use rsmpeg::avutil::AVMem;
use rsmpeg::ffi;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{config, file};

pub struct Sink {
    tx: Sender<Vec<u8>>,
    buffer_size: usize,
    format: format::Transcode,
}

impl Sink {
    pub fn new(
        config: &config::Transcode,
        property: file::Property<format::Transcode>,
    ) -> (Self, Receiver<Vec<u8>>) {
        let (tx, rx) = tokio::sync::mpsc::channel(config.channel_size);
        (Self { tx, buffer_size: config.buffer_size, format: property.format }, rx)
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

        match self.tx.blocking_send(data) {
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
