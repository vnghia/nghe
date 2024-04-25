use anyhow::Result;
use lofty::file::FileType;

pub trait GenericPath {
    // Path
    fn relative_path(&self) -> &str;

    // Data
    async fn read(&self) -> Result<Vec<u8>>;
    async fn read_lrc(&self) -> Result<String>;

    // Metadata
    fn size(&self) -> u64;
    fn file_type(&self) -> FileType;
}
