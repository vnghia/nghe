use anyhow::Result;
use lofty::file::FileType;

#[cfg(test)]
use crate::utils::test::TemporaryFsRoot;

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub size: u32,
}

pub trait PathTrait {
    const PATH_SEPARATOR: &'static str;

    fn relative(&self, base: &str) -> &str;

    fn file_type(&self) -> FileType;
    async fn metadata(&self) -> Result<Metadata>;

    async fn read(&self) -> Result<Vec<u8>>;
    async fn read_to_string(&self) -> Result<String>;

    fn lrc(&self) -> Self;
    async fn read_lrc(&self) -> Result<String>;
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct AbsolutePath<P: PathTrait> {
    pub absolute_path: P,
    pub relative_path: String,
    pub metadata: Metadata,
}

impl<P: PathTrait> AbsolutePath<P> {
    pub fn new(base: &str, absolute_path: P, metadata: Metadata) -> Self {
        let relative_path = absolute_path.relative(base).to_string();
        Self { absolute_path, relative_path, metadata }
    }

    pub fn file_type(&self) -> FileType {
        self.absolute_path.file_type()
    }

    pub async fn metadata(&self) -> Result<Metadata> {
        self.absolute_path.metadata().await
    }

    pub async fn read(&self) -> Result<Vec<u8>> {
        self.absolute_path.read().await
    }

    pub async fn read_lrc(&self) -> Result<String> {
        self.absolute_path.read_lrc().await
    }
}

#[cfg(test)]
pub trait PathTest {
    fn ext(&self) -> &str;

    async fn write<D: AsRef<[u8]>>(&self, data: D);
    async fn delete(&self);
    async fn mkdir(&self);

    fn new(root: &TemporaryFsRoot, path: Option<&str>) -> Self;
    fn new_self(&self, root: &TemporaryFsRoot, path: Option<&str>) -> Self;
    fn join(&self, path: &str) -> Self;
    fn with_ext(&self, ext: &str) -> Self;
}
