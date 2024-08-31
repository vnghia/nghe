use std::borrow::Borrow;

use nghe_api::constant;
use tempfile::{Builder, TempDir};
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::filesystem::{self, local};

#[derive(Debug)]
pub struct Mock {
    root: TempDir,
    filesystem: local::Filesystem,
}

impl Borrow<local::Filesystem> for Mock {
    fn borrow(&self) -> &local::Filesystem {
        &self.filesystem
    }
}

impl Borrow<local::Filesystem> for &Mock {
    fn borrow(&self) -> &local::Filesystem {
        &self.filesystem
    }
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

impl<B: Borrow<Mock> + filesystem::Trait> super::MockTrait for B {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        self.borrow().root.path().to_str().unwrap().into()
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        let path = self.borrow().prefix().join(path);
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
