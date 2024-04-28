use std::path::Path;

use anyhow::Result;

use crate::OSError;

pub async fn check_dir<P: AsRef<Path>>(path: P) -> Result<P> {
    if path.as_ref().is_absolute() && tokio::fs::metadata(&path).await?.is_dir() {
        Ok(path)
    } else {
        anyhow::bail!(OSError::InvalidParameter("path is not absolute or not a directory".into()))
    }
}
