use typed_path::{Utf8PlatformPath, Utf8PlatformPathBuf};

use crate::file::audio;

pub fn dir() -> Utf8PlatformPathBuf {
    Utf8PlatformPath::new(&env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("assets").join("test")
}

pub fn path(format: audio::Format) -> Utf8PlatformPathBuf {
    dir().join("sample").with_extension(format.as_ref())
}

pub fn transcoded(format: nghe_api::common::format::Transcode, offset: u32) -> Utf8PlatformPathBuf {
    dir()
        .join("transcoded")
        .join(concat_string::concat_string!("sample-", offset.to_string()))
        .with_extension(format.as_ref())
}
