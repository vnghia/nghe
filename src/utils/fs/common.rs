use anyhow::Result;

use crate::utils::path::PathMetadata;

#[async_trait::async_trait]
pub trait FsTrait {
    fn strip_prefix<'a>(&self, path: &'a str, base: &str) -> &'a str;
    fn ext<'a>(&self, path: &'a str) -> &'a str;
    fn with_ext(&self, path: &str, ext: &str) -> String;

    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    async fn read_to_string(&self, path: &str) -> Result<String>;
    async fn metadata(&self, path: &str) -> Result<PathMetadata>;
}
