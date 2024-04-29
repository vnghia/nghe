use anyhow::Result;
use concat_string::concat_string;
use nghe_types::constant::SERVER_NAME;
use tempfile::{Builder, TempDir};
use typed_path::{Utf8NativeEncoding, Utf8Path};

use super::TemporaryFsTrait;
use crate::utils::fs::{FsTrait, LocalFs};
use crate::utils::path::PathMetadata;

pub struct TemporaryLocalFs {
    pub root: TempDir,
    pub fs: LocalFs,
}

impl TemporaryLocalFs {
    pub fn new() -> Self {
        Self {
            root: Builder::new().prefix(&concat_string!(SERVER_NAME, ".")).tempdir().unwrap(),
            fs: LocalFs,
        }
    }
}

#[async_trait::async_trait]
impl FsTrait for TemporaryLocalFs {
    fn strip_prefix<'a>(&self, path: &'a str, base: &str) -> &'a str {
        self.fs.strip_prefix(path, base)
    }

    fn ext<'a>(&self, path: &'a str) -> &'a str {
        self.fs.ext(path)
    }

    fn with_ext(&self, path: &str, ext: &str) -> String {
        self.fs.with_ext(path, ext)
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        self.fs.read(path).await
    }

    async fn read_to_string(&self, path: &str) -> Result<String> {
        self.fs.read_to_string(path).await
    }

    async fn metadata(&self, path: &str) -> Result<PathMetadata> {
        self.fs.metadata(path).await
    }
}

#[async_trait::async_trait]
impl TemporaryFsTrait for TemporaryLocalFs {
    fn join(&self, base: &str, path: &str) -> String {
        Utf8Path::<Utf8NativeEncoding>::new(base).join(path).into_string()
    }

    fn prefix(&self) -> &str {
        self.root.path().to_str().unwrap()
    }

    async fn mkdir(&self, path: &str) {
        tokio::fs::create_dir_all(path).await.unwrap();
    }

    async fn write(&self, path: &str, data: &[u8]) {
        self.mkdir(Utf8Path::<Utf8NativeEncoding>::new(path).parent().unwrap().as_str()).await;
        tokio::fs::write(path, data).await.unwrap();
    }

    async fn remove(&self, path: &str) {
        tokio::fs::remove_file(path).await.unwrap();
    }
}

impl Default for TemporaryLocalFs {
    fn default() -> Self {
        Self::new()
    }
}
