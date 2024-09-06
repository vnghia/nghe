use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::filesystem::Trait;
use crate::Error;

#[derive(Debug)]
pub enum MockImpl<'fs> {
    Local(&'fs super::local::Mock),
    S3(&'fs super::s3::Mock),
}

pub trait MockTrait: Trait {
    fn prefix(&self) -> Utf8TypedPath<'_>;

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf;
    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]);
    async fn delete(&self, path: Utf8TypedPath<'_>);
}

impl<'fs> Trait for MockImpl<'fs> {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        match self {
            MockImpl::Local(filesystem) => filesystem.check_folder(path).await,
            MockImpl::S3(filesystem) => filesystem.check_folder(path).await,
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        match self {
            MockImpl::Local(filesystem) => filesystem.read(path).await,
            MockImpl::S3(filesystem) => filesystem.read(path).await,
        }
    }
}

impl<'fs> MockTrait for MockImpl<'fs> {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        match self {
            MockImpl::Local(filesystem) => filesystem.prefix(),
            MockImpl::S3(filesystem) => filesystem.prefix(),
        }
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        match self {
            MockImpl::Local(filesystem) => filesystem.create_dir(path).await,
            MockImpl::S3(filesystem) => filesystem.create_dir(path).await,
        }
    }

    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]) {
        match self {
            MockImpl::Local(filesystem) => filesystem.write(path, data).await,
            MockImpl::S3(filesystem) => filesystem.write(path, data).await,
        }
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        match self {
            MockImpl::Local(filesystem) => filesystem.delete(path).await,
            MockImpl::S3(filesystem) => filesystem.delete(path).await,
        }
    }
}
