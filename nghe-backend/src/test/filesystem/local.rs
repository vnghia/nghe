use std::borrow::Cow;

use nghe_api::constant;
use tempfile::{Builder, TempDir};
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::filesystem::{self, local};
use crate::Error;

#[derive(Debug)]
pub struct Mock {
    root: TempDir,
    filesystem: local::Filesystem,
}

impl Mock {
    pub fn new(filesystem: local::Filesystem) -> Self {
        Self {
            root: Builder::new()
                .prefix(&const_format::concatc!(constant::SERVER_NAME, "."))
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

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        self.filesystem.read(path).await
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
        let path = self.prefix().join(path);
        tokio::fs::create_dir_all(path.as_str()).await.unwrap();
        path
    }

    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]) {
        self.create_dir(path.parent().unwrap()).await;
        tokio::fs::write(path.as_str(), data).await.unwrap();
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        tokio::fs::remove_file(path.as_str()).await.unwrap();
    }
}
