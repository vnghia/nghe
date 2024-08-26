use crate::filesystem;
use crate::orm::music_folders::FilesystemType;

#[derive(Debug, Clone)]
pub struct Filesystem {
    local: filesystem::local::Filesystem,
}

impl Filesystem {
    pub fn to_impl(&self, filesystem_type: FilesystemType) -> filesystem::Impl<'_> {
        match filesystem_type {
            FilesystemType::Local => (&self.local).into(),
            FilesystemType::S3 => todo!(),
        }
    }
}
