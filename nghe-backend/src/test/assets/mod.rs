use lofty::file::FileType;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

pub fn ext(file_type: FileType) -> &'static str {
    match file_type {
        FileType::Flac => "flac",
        _ => unreachable!(),
    }
}

pub fn dir() -> Utf8TypedPathBuf {
    Utf8TypedPath::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("assets").join("test")
}

pub fn path(file_type: FileType) -> Utf8TypedPathBuf {
    dir().join("sample").with_extension(ext(file_type))
}
