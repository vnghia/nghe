use std::path::PathBuf;

use concat_string::concat_string;
use lofty::FileType;

use crate::utils::song::file_type::to_extension;

pub fn get_media_asset_path(file_type: &FileType) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("test")
        .join(concat_string!("sample.", to_extension(file_type)))
}
