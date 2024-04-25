use std::fs::Metadata;
use std::path::{Path, PathBuf};

use anyhow::Result;
use lofty::file::FileType;

use super::generic::GenericPath;

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct LocalPath {
    pub absolute_path: PathBuf,
    pub relative_path: String,
    pub metadata: Metadata,
}

impl LocalPath {
    pub fn new<P: AsRef<Path>>(root: P, absolute_path: PathBuf, metadata: Metadata) -> Self {
        let relative_path = absolute_path
            .strip_prefix(&root)
            .expect("this path should always contains the root path")
            .to_str()
            .expect("non utf-8 path encountered")
            .to_string();
        Self { absolute_path, relative_path, metadata }
    }
}

impl GenericPath for LocalPath {
    // Path
    fn relative_path(&self) -> &str {
        &self.relative_path
    }

    // Data
    async fn read(&self) -> Result<Vec<u8>> {
        tokio::fs::read(&self.absolute_path).await.map_err(anyhow::Error::from)
    }

    async fn read_lrc(&self) -> Result<String> {
        tokio::fs::read_to_string(&self.absolute_path.with_extension("lrc"))
            .await
            .map_err(anyhow::Error::from)
    }

    // Metadata
    fn size(&self) -> u64 {
        self.metadata.len()
    }

    fn file_type(&self) -> FileType {
        FileType::from_path(&self.absolute_path).expect("this should not happen")
    }
}
