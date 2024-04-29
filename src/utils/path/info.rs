#[derive(Debug, Clone, Copy)]
pub struct PathMetadata {
    pub size: u32,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone))]
pub struct PathInfo {
    pub path: String,
    pub metadata: PathMetadata,
}

impl PathInfo {
    pub fn new<P: Into<String>, M: Into<PathMetadata>>(path: P, metadata: M) -> Self {
        Self { path: path.into(), metadata: metadata.into() }
    }
}
