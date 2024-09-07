use tokio::sync::mpsc::Sender;
use typed_path::Utf8TypedPath;

use super::Entry;
use crate::Error;

#[derive(Debug)]
pub enum Impl<'fs> {
    Local(&'fs super::local::Filesystem),
    S3(&'fs super::s3::Filesystem),
}

pub trait Trait {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error>;
    async fn list_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<Entry>,
    ) -> Result<(), Error>;

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error>;
}

impl<'fs> Trait for Impl<'fs> {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.check_folder(path).await,
            Impl::S3(filesystem) => filesystem.check_folder(path).await,
        }
    }

    async fn list_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<Entry>,
    ) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.list_folder(path, minimum_size, tx).await,
            Impl::S3(filesystem) => filesystem.list_folder(path, minimum_size, tx).await,
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read(path).await,
            Impl::S3(filesystem) => filesystem.read(path).await,
        }
    }
}
