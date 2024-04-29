use std::fs::Metadata;

use anyhow::Result;
use typed_path::{Utf8NativeEncoding, Utf8Path};

use super::FsTrait;
use crate::utils::path::PathMetadata;

pub struct LocalFs;

impl From<Metadata> for PathMetadata {
    fn from(value: std::fs::Metadata) -> Self {
        Self { size: value.len() as _ }
    }
}

#[async_trait::async_trait]
impl FsTrait for LocalFs {
    fn strip_prefix<'a>(&self, path: &'a str, base: &str) -> &'a str {
        Utf8Path::<Utf8NativeEncoding>::new(path)
            .strip_prefix(base)
            .expect("path should be a children of base")
            .as_str()
    }

    fn ext<'a>(&self, path: &'a str) -> &'a str {
        Utf8Path::<Utf8NativeEncoding>::new(path)
            .extension()
            .expect("path should have an extension")
    }

    fn with_ext(&self, path: &str, ext: &str) -> String {
        Utf8Path::<Utf8NativeEncoding>::new(path).with_extension(ext).into_string()
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        tokio::fs::read(path).await.map_err(anyhow::Error::from)
    }

    async fn read_to_string(&self, path: &str) -> Result<String> {
        tokio::fs::read_to_string(path).await.map_err(anyhow::Error::from)
    }

    async fn metadata(&self, path: &str) -> Result<PathMetadata> {
        tokio::fs::metadata(path).await.map(PathMetadata::from).map_err(anyhow::Error::from)
    }
}
