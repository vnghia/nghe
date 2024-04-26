use anyhow::Result;
use lofty::file::FileType;

#[derive(Debug, Clone, Copy)]
pub struct Metadata {
    pub is_dir: bool,
    pub size: u64,
}

pub trait PathMetadata {
    fn file_type(&self) -> FileType;
    async fn metadata(&self) -> Result<Metadata>;
}

pub trait PathRead {
    async fn read(&self) -> Result<Vec<u8>>;
    async fn read_to_string(&self) -> Result<String>;
}

pub trait PathLrc {
    fn lrc(&self) -> Self;
}

pub trait PathRelative {
    fn relative(&self, base: &str) -> String;
}

pub trait PathTrait = PathRead + PathLrc + PathRelative + PathMetadata;

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct AbsolutePath<P: PathTrait> {
    pub absolute_path: P,
    pub relative_path: String,
    pub metadata: Metadata,
}

impl<P: PathTrait> AbsolutePath<P> {
    pub fn new(base: &str, absolute_path: P, metadata: Metadata) -> Self {
        let relative_path = absolute_path.relative(base);
        Self { absolute_path, relative_path, metadata }
    }
}

impl<P: PathTrait> PathMetadata for AbsolutePath<P> {
    fn file_type(&self) -> FileType {
        self.absolute_path.file_type()
    }

    async fn metadata(&self) -> Result<Metadata> {
        self.absolute_path.metadata().await
    }
}

impl<P: PathTrait> PathRead for AbsolutePath<P> {
    async fn read(&self) -> Result<Vec<u8>> {
        self.absolute_path.read().await
    }

    async fn read_to_string(&self) -> Result<String> {
        self.absolute_path.read_to_string().await
    }
}
