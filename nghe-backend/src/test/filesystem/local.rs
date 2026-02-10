use std::borrow::Cow;

use nghe_api::constant;
use tempfile::{Builder, TempDir};
#[cfg(not(target_os = "linux"))]
use tokio::fs;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};
#[cfg(target_os = "linux")]
use uring_file::fs;

use crate::Error;
use crate::file::{self, audio};
use crate::filesystem::{self, local};
use crate::http::binary;
#[derive(Debug)]
pub struct Mock {
    root: TempDir,
    filesystem: local::Filesystem,
}

impl Mock {
    pub fn new(filesystem: local::Filesystem) -> Self {
        Self {
            root: Builder::new()
                .prefix(&const_format::concatcp!(constant::SERVER_NAME, "."))
                .tempdir()
                .unwrap(),
            filesystem,
        }
    }
}

impl filesystem::Trait for Mock {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        self.filesystem.check_folder(path).await
    }

    async fn scan_folder(
        &self,
        sender: filesystem::entry::Sender,
        prefix: Utf8TypedPath<'_>,
    ) -> Result<(), Error> {
        self.filesystem.scan_folder(sender, prefix).await
    }

    async fn exists(&self, path: Utf8TypedPath<'_>) -> Result<bool, Error> {
        self.filesystem.exists(path).await
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        self.filesystem.read(path).await
    }

    async fn read_to_string(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        self.filesystem.read_to_string(path).await
    }

    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error> {
        self.filesystem.read_to_binary(source, offset).await
    }

    async fn transcode_input(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        self.filesystem.transcode_input(path).await
    }
}

impl super::Trait for Mock {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        self.root.path().to_str().unwrap().into()
    }

    fn main(&self) -> filesystem::Impl<'_> {
        filesystem::Impl::Local(Cow::Borrowed(&self.filesystem))
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        let path = self.absolutize(path);
        fs::create_dir_all(path.as_str()).await.unwrap();
        path
    }

    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]) {
        let path = self.absolutize(path);
        self.create_dir(path.parent().unwrap()).await;
        fs::write(path.as_str(), data).await.unwrap();
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        let path = self.absolutize(path);
        fs::remove_file(path.as_str()).await.unwrap();
    }
}
