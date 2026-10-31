use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::Error;
use crate::filesystem::{self, entry};

#[derive(Debug)]
pub enum Impl<'fs> {
    Local(&'fs super::local::Mock),
    S3(&'fs super::s3::Mock),
}

impl Impl<'_> {
    pub fn path(&self) -> filesystem::path::Builder {
        self.main().path()
    }

    pub fn fake_path(&self, depth: usize) -> Utf8TypedPathBuf {
        fake::vec![String; depth + 1]
            .into_iter()
            .fold(self.path().empty(), |path, component| path.join(component))
    }
}

pub trait Trait: filesystem::Trait {
    fn prefix(&self) -> Utf8TypedPath<'_>;
    fn main(&self) -> filesystem::Impl<'_>;

    fn absolutize(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        if path.is_absolute() { path.to_path_buf() } else { self.prefix().join(path) }
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf;
    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]);
    async fn delete(&self, path: Utf8TypedPath<'_>);
}

impl filesystem::Trait for Impl<'_> {
    type EntryStream = entry::Stream;

    type AsyncRead = filesystem::io::AsyncRead;
    type Read = filesystem::io::Read;

    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.check_folder(path).await,
            Impl::S3(filesystem) => filesystem.check_folder(path).await,
        }
    }

    fn scan_folder(
        &self,
        prefix: Utf8TypedPath<'_>,
        minimum_size: usize,
    ) -> Result<Self::EntryStream, Error> {
        match self {
            Impl::Local(filesystem) => filesystem
                .scan_folder(prefix, minimum_size)
                .map(|stream| entry::Stream::Local(Box::pin(stream))),
            Impl::S3(filesystem) => {
                filesystem.scan_folder(prefix, minimum_size).map(entry::Stream::S3)
            }
        }
    }

    async fn exists(&self, path: Utf8TypedPath<'_>) -> Result<bool, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.exists(path).await,
            Impl::S3(filesystem) => filesystem.exists(path).await,
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read(path).await,
            Impl::S3(filesystem) => filesystem.read(path).await,
        }
    }

    async fn read_to_string(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read_to_string(path).await,
            Impl::S3(filesystem) => filesystem.read_to_string(path).await,
        }
    }

    async fn read_to_async_read(
        &self,
        path: Utf8TypedPath<'_>,
        offset: Option<u64>,
    ) -> Result<Self::AsyncRead, Error> {
        match self {
            Impl::Local(filesystem) => {
                filesystem.read_to_async_read(path, offset).await.map(Into::into)
            }
            Impl::S3(filesystem) => {
                filesystem.read_to_async_read(path, offset).await.map(filesystem::io::AsyncRead::S3)
            }
        }
    }

    async fn read_to_read(
        &self,
        path: Utf8TypedPath<'_>,
        offset: Option<u64>,
    ) -> Result<Self::Read, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read_to_read(path, offset).await.map(Into::into),
            Impl::S3(filesystem) => {
                filesystem.read_to_read(path, offset).await.map(filesystem::io::Read::S3)
            }
        }
    }
}

impl Trait for Impl<'_> {
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
