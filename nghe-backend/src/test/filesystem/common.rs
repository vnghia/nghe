use tokio::sync::mpsc::Sender;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::{filesystem, Error};

#[derive(Debug)]
pub enum Impl<'fs> {
    Local(&'fs super::local::Mock),
    S3(&'fs super::s3::Mock),
}

pub trait Trait: filesystem::Trait {
    fn prefix(&self) -> Utf8TypedPath<'_>;
    fn main(&self) -> filesystem::Impl<'_>;

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf;
    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]);
    async fn delete(&self, path: Utf8TypedPath<'_>);
}

impl<'fs> filesystem::Trait for Impl<'fs> {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.check_folder(path).await,
            Impl::S3(filesystem) => filesystem.check_folder(path).await,
        }
    }

    async fn scan_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<filesystem::Entry>,
    ) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.scan_folder(path, minimum_size, tx).await,
            Impl::S3(filesystem) => filesystem.scan_folder(path, minimum_size, tx).await,
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read(path).await,
            Impl::S3(filesystem) => filesystem.read(path).await,
        }
    }
}

impl<'fs> Trait for Impl<'fs> {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        match self {
            Impl::Local(filesystem) => filesystem.prefix(),
            Impl::S3(filesystem) => filesystem.prefix(),
        }
    }

    fn main(&self) -> filesystem::Impl<'_> {
        match self {
            Impl::Local(filesystem) => filesystem.main(),
            Impl::S3(filesystem) => filesystem.main(),
        }
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        match self {
            Impl::Local(filesystem) => filesystem.create_dir(path).await,
            Impl::S3(filesystem) => filesystem.create_dir(path).await,
        }
    }

    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]) {
        match self {
            Impl::Local(filesystem) => filesystem.write(path, data).await,
            Impl::S3(filesystem) => filesystem.write(path, data).await,
        }
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        match self {
            Impl::Local(filesystem) => filesystem.delete(path).await,
            Impl::S3(filesystem) => filesystem.delete(path).await,
        }
    }
}
