use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::OSError;

pub async fn check_dir<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
    let path = tokio::fs::canonicalize(path).await?;
    if tokio::fs::metadata(&path).await?.is_dir() {
        Ok(path)
    } else {
        anyhow::bail!(OSError::InvalidParameter("path is not a directory".into()))
    }
}
