mod format;
mod sink;
mod transcoder;

pub use sink::Sink;
pub use transcoder::Transcoder;
use typed_path::Utf8PlatformPathBuf;

#[derive(Debug)]
pub struct Path {
    pub input: String,
    pub output: Option<Utf8PlatformPathBuf>,
}
