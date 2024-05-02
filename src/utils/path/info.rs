use time::OffsetDateTime;
use typed_path::Utf8PathBuf;

use crate::utils::fs::FsTrait;

#[derive(Debug, Clone, Copy)]
pub struct PathMetadata {
    pub size: u32,
    pub last_modified: Option<OffsetDateTime>,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct PathInfo<Fs: FsTrait> {
    pub path: Utf8PathBuf<Fs::E>,
    pub metadata: PathMetadata,
}

impl<Fs: FsTrait> PathInfo<Fs> {
    pub fn new<P: Into<Utf8PathBuf<Fs::E>>, M: Into<PathMetadata>>(path: P, metadata: M) -> Self {
        Self { path: path.into(), metadata: metadata.into() }
    }
}
