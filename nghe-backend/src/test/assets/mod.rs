use typed_path::Utf8TypedPathBuf;

use crate::filesystem::path;
use crate::media::audio;

pub fn dir() -> Utf8TypedPathBuf {
    path::Local::from_str(&env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("assets").join("test")
}

pub fn path(ty: audio::Type) -> Utf8TypedPathBuf {
    dir().join("sample").with_extension(ty.as_ref())
}
