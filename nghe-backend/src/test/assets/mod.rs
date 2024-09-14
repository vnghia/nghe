use nghe_api::common::filesystem;
use typed_path::Utf8TypedPathBuf;

use crate::filesystem::path;
use crate::media::file;

pub fn dir() -> Utf8TypedPathBuf {
    path::Local::from_str(&env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("assets").join("test")
}

pub fn path(file_type: file::Type) -> Utf8TypedPathBuf {
    dir().join("sample").with_extension(file_type.as_ref())
}
