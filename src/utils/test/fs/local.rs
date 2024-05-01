use std::any::Any;

use anyhow::Result;
use concat_string::concat_string;
use nghe_types::constant::SERVER_NAME;
use tempfile::{Builder, TempDir};
use typed_path::Utf8Path;

use super::{extension, join, strip_prefix, with_extension, TemporaryFsTrait};
use crate::config::ScanConfig;
use crate::utils::fs::{FsTrait, LocalFs};

pub struct TemporaryLocalFs {
    pub root: TempDir,
    pub fs: LocalFs,
}

impl TemporaryLocalFs {
    pub fn new() -> Self {
        let scan_config = ScanConfig::default();
        Self {
            root: Builder::new().prefix(&concat_string!(SERVER_NAME, ".")).tempdir().unwrap(),
            fs: LocalFs { scan_parallel: scan_config.parallel },
        }
    }
}

#[async_trait::async_trait]
impl TemporaryFsTrait for TemporaryLocalFs {
    fn prefix(&self) -> &str {
        self.root.path().to_str().unwrap()
    }

    fn fs(&self) -> &dyn Any {
        &self.fs
    }

    fn join(&self, base: &str, path: &str) -> String {
        join::<LocalFs>(base, path)
    }

    fn strip_prefix<'a>(&self, path: &'a str, base: &str) -> &'a str {
        strip_prefix::<LocalFs>(path, base)
    }

    fn extension<'a>(&self, path: &'a str) -> &'a str {
        extension::<LocalFs>(path)
    }

    fn with_extension(&self, path: &str, extension: &str) -> String {
        with_extension::<LocalFs>(path, extension)
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        self.fs.read(path).await
    }

    async fn read_to_string(&self, path: &str) -> Result<String> {
        self.fs.read_to_string(path).await
    }

    async fn mkdir(&self, path: &str) {
        tokio::fs::create_dir_all(path).await.unwrap();
    }

    async fn write(&self, path: &str, data: &[u8]) {
        self.mkdir(Utf8Path::<<LocalFs as FsTrait>::E>::new(path).parent().unwrap().as_str()).await;
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
