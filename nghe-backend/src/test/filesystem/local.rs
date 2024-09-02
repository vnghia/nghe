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
}

impl filesystem::Trait for &Mock {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        self.filesystem.check_folder(path).await
    }
}

impl super::MockTrait for Mock {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        self.root.path().to_str().unwrap().into()
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        let path = self.prefix().join(path);
        tokio::fs::create_dir_all(path.as_str()).await.unwrap();
        path
    }

    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]) {
        let path = path.as_str();
        self.create_dir(path.into()).await;
        tokio::fs::write(path, data).await.unwrap();
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        tokio::fs::remove_file(path.as_str()).await.unwrap();
    }
}

impl super::MockTrait for &Mock {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        (*self).prefix()
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        (*self).create_dir(path).await
    }

    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]) {
        (*self).write(path, data).await;
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        (*self).delete(path).await;
    }
}
