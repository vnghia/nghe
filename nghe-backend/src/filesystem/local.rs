use typed_path::Utf8TypedPath;

use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct Filesystem;

impl super::Trait for Filesystem {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        if path.is_absolute() && tokio::fs::metadata(path.as_str()).await?.is_dir() {
            Ok(())
        } else {
            Err(Error::InvalidParameter("Folder path must be absolute and be a directory"))
        }
    }
}

impl super::Trait for &Filesystem {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        (*self).check_folder(path).await
    }
}
