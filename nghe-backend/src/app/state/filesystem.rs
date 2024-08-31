use nghe_api::music_folder::FilesystemType;

use crate::filesystem::{local, Impl};

#[derive(Debug, Clone)]
pub struct Filesystem {
    local: local::Filesystem,
}

impl Filesystem {
    pub fn new() -> Self {
        let local = local::Filesystem;
        Self { local }
    }

    pub fn to_impl(&self, filesystem_type: FilesystemType) -> Impl<'_> {
        match filesystem_type {
            FilesystemType::Local | FilesystemType::S3 => (&self.local).into(),
        }
    }
}
