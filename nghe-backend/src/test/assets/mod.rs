use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::media::file;

pub fn ext(file_type: file::Type) -> &'static str {
    match file_type {
        file::Type::Flac => "flac",
    }
}

pub fn dir() -> Utf8TypedPathBuf {
    Utf8TypedPath::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("assets").join("test")
}

pub fn path(file_type: file::Type) -> Utf8TypedPathBuf {
    dir().join("sample").with_extension(ext(file_type))
}
