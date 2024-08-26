use crate::filesystem::{local, Impl};
use crate::orm::music_folders::FilesystemType;

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
            FilesystemType::Local => (&self.local).into(),
            FilesystemType::S3 => todo!(),
        }
    }
}
