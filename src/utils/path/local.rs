use std::borrow::Cow;
use std::path::Path;

use anyhow::Result;
use lofty::file::FileType;

use super::{Metadata, PathLrc, PathMetadata, PathRead, PathRelative};

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct LocalPath<'a> {
    pub path: Cow<'a, Path>,
}

impl From<std::fs::Metadata> for Metadata {
    fn from(value: std::fs::Metadata) -> Self {
        Self { is_dir: value.is_dir(), size: value.len() }
    }
}

impl<'a> PathMetadata for LocalPath<'a> {
    fn file_type(&self) -> FileType {
        FileType::from_path(&self.path).expect("file type is none which is impossible")
    }

    async fn metadata(&self) -> Result<Metadata> {
        tokio::fs::metadata(&self.path).await.map(Metadata::from).map_err(anyhow::Error::from)
    }
}

impl<'a> PathRead for LocalPath<'a> {
    async fn read(&self) -> Result<Vec<u8>> {
        tokio::fs::read(&self.path).await.map_err(anyhow::Error::from)
    }

    async fn read_to_string(&self) -> Result<String> {
        tokio::fs::read_to_string(&self.path).await.map_err(anyhow::Error::from)
    }
}

impl<'a> PathLrc for LocalPath<'a> {
    fn lrc(&self) -> Self {
        Self { path: self.path.with_extension("lrc").into() }
    }
}

impl<'a> PathRelative for LocalPath<'a> {
    fn relative(&self, base: &str) -> String {
        self.path
            .strip_prefix(base)
            .expect("this path should always contains base path")
            .to_str()
            .expect("non utf-8 path encountered")
            .to_string()
    }
}
