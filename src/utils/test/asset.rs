use concat_string::concat_string;
use lofty::file::FileType;

use crate::utils::fs::{LocalPath, LocalPathBuf};
use crate::utils::song::file_type::to_extension;

pub fn get_asset_dir() -> LocalPathBuf {
    LocalPath::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}

pub fn get_media_asset_path(file_type: &FileType) -> LocalPathBuf {
    get_asset_dir().join("test").join(concat_string!("sample.", to_extension(file_type)))
}
