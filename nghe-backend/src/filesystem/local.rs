use std::borrow::Borrow;

use typed_path::Utf8TypedPath;

use crate::Error;

#[derive(Debug, Clone)]
pub struct Filesystem;

impl<B: Borrow<Filesystem>> super::Trait for B {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        if path.is_absolute() && tokio::fs::metadata(path.as_str()).await?.is_dir() {
            Ok(())
        } else {
            Err(Error::InvalidParameter("Folder path must be absolute and be a directory"))
        }
    }
}
