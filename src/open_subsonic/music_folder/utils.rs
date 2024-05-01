use anyhow::Result;

use crate::utils::fs::LocalPath;
use crate::OSError;

pub async fn check_dir<P: AsRef<LocalPath>>(path: P) -> Result<P> {
    if path.as_ref().is_absolute() && tokio::fs::metadata(path.as_ref().as_str()).await?.is_dir() {
        Ok(path)
    } else {
        anyhow::bail!(OSError::InvalidParameter("path is not absolute or not a directory".into()))
    }
}
