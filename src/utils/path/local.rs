use std::borrow::Cow;
use std::fmt::Display;
use std::path::Path;

use anyhow::Result;
use lofty::file::FileType;

#[cfg(test)]
use super::PathTest;
use super::{Metadata, PathTrait};
#[cfg(test)]
use crate::utils::test::TemporaryFsRoot;

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct LocalPath<'a> {
    pub path: Cow<'a, Path>,
}

impl From<std::fs::Metadata> for Metadata {
    fn from(value: std::fs::Metadata) -> Self {
        Self { size: value.len() as _ }
    }
}

impl<'a> PathTrait for LocalPath<'a> {
    const PATH_SEPARATOR: &'static str = std::path::MAIN_SEPARATOR_STR;

    fn relative(&self, base: &str) -> &str {
        self.path
            .strip_prefix(base)
            .expect("this path should always contains base path")
            .to_str()
            .expect("non utf-8 path encountered")
    }

    fn file_type(&self) -> FileType {
        FileType::from_path(&self.path).expect("file type is none which is impossible")
    }

    async fn metadata(&self) -> Result<Metadata> {
        tokio::fs::metadata(&self.path).await.map(Metadata::from).map_err(anyhow::Error::from)
    }

    async fn read(&self) -> Result<Vec<u8>> {
        tokio::fs::read(&self.path).await.map_err(anyhow::Error::from)
    }

    async fn read_to_string(&self) -> Result<String> {
        tokio::fs::read_to_string(&self.path).await.map_err(anyhow::Error::from)
    }

    fn lrc(&self) -> Self {
        Self { path: self.path.with_extension("lrc").into() }
    }

    async fn read_lrc(&self) -> Result<String> {
        self.lrc().read_to_string().await
    }
}

impl<'a, P: Into<Cow<'a, Path>>> From<P> for LocalPath<'a> {
    fn from(value: P) -> Self {
        Self { path: value.into() }
    }
}

impl<'a> Display for LocalPath<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_str().expect("non utf-8 path encountered"))
    }
}

#[cfg(test)]
impl<'a> PathTest for LocalPath<'a> {
    fn ext(&self) -> &str {
        self.path.extension().unwrap().to_str().unwrap()
    }

    async fn write<D: AsRef<[u8]>>(&self, data: D) {
        tokio::fs::create_dir_all(&self.path.parent().unwrap()).await.unwrap();
        tokio::fs::write(&self.path, data).await.unwrap();
    }

    async fn delete(&self) {
        tokio::fs::remove_file(&self.path).await.unwrap();
    }

    async fn mkdir(&self) {
        tokio::fs::create_dir_all(&self.path).await.unwrap();
    }

    fn new(root: &TemporaryFsRoot, path: Option<&str>) -> Self {
        if let Some(path) = path {
            Path::new(path).to_path_buf().into()
        } else {
            root.local.path().to_path_buf().into()
        }
    }

    fn new_self(&self, root: &TemporaryFsRoot, path: Option<&str>) -> Self {
        Self::new(root, path)
    }

    fn join(&self, path: &str) -> Self {
        self.path.join(path).into()
    }

    fn with_ext(&self, ext: &str) -> Self {
        self.path.with_extension(ext).into()
    }
}
