use crate::utils::song::file_type::to_extension;

use concat_string::concat_string;
use lofty::FileType;
use std::path::PathBuf;

pub fn get_media_asset_path(file_type: &FileType) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("test")
        .join(concat_string!("sample.", to_extension(file_type)))
}

pub const fn get_media_asset_duration(file_type: &FileType) -> i32 {
    match file_type {
        FileType::Flac => 15,
        FileType::Mpeg => 4,
        _ => unimplemented!(),
    }
}
