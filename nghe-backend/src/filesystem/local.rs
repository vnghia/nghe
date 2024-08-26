use typed_path::Utf8TypedPath;

use crate::Error;

pub struct Filesystem;

impl super::Trait for Filesystem {
    async fn check_folder<'a>(&self, path: Utf8TypedPath<'a>) -> Result<Utf8TypedPath<'a>, Error> {
        if path.is_absolute() && tokio::fs::metadata(path.as_str()).await?.is_dir() {
            Ok(path)
        } else {
            Err(Error::InvalidParameter("Folder path must be absolute and be a directory"))
        }
    }
}
