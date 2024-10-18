use typed_path::Utf8TypedPathBuf;

use crate::file::audio;
use crate::filesystem::path;

pub fn dir() -> Utf8TypedPathBuf {
    path::Local::from_str(&env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("assets").join("test")
}

pub fn path(format: audio::Format) -> Utf8TypedPathBuf {
    dir().join("sample").with_extension(format.as_ref())
}
