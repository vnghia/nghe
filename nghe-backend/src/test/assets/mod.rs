use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::media::file;

pub fn dir() -> Utf8TypedPathBuf {
    Utf8TypedPath::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("assets").join("test")
}

pub fn path(file_type: file::Type) -> Utf8TypedPathBuf {
    dir().join("sample").with_extension(file_type.as_ref())
}
