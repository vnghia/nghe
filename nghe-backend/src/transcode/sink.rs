use std::ffi::CStr;
use std::io::Write;

use educe::Educe;
use loole::Sender;
use nghe_api::common::format;
use rsmpeg::avformat::{AVIOContextContainer, AVIOContextCustom};
use rsmpeg::avutil::AVMem;
use rsmpeg::ffi;

#[derive(Educe)]
#[educe(Debug)]
pub struct Sink {
    #[educe(Debug(ignore))]
    pub tx: Sender<Vec<u8>>,
    pub buffer_size: usize,
    pub format: format::Transcode,
    pub file: Option<std::fs::File>,
}

impl Sink {
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
