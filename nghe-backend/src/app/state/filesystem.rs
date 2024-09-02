use color_eyre::eyre::OptionExt;
use nghe_api::music_folder::FilesystemType;

use crate::filesystem::{local, s3, Impl};
use crate::{config, Error};

#[derive(Debug, Clone)]
pub struct Filesystem {
    local: local::Filesystem,
    s3: Option<s3::Filesystem>,
}

impl Filesystem {
    pub async fn new(tls: &config::filesystem::Tls, s3: &config::filesystem::S3) -> Self {
        let local = local::Filesystem;
        let s3 = if s3.enable { Some(s3::Filesystem::new(tls, s3).await) } else { None };
        Self { local, s3 }
    }

    pub fn to_impl(&self, filesystem_type: FilesystemType) -> Result<Impl<'_>, Error> {
        Ok(match filesystem_type {
            FilesystemType::Local => (&self.local).into(),
            FilesystemType::S3 => {
                self.s3.as_ref().ok_or_eyre("S3 filesystem is not enabled")?.into()
            }
        })
    }

    #[cfg(test)]
    pub fn local(&self) -> local::Filesystem {
        self.local
    }

    #[cfg(test)]
    pub fn s3(&self) -> s3::Filesystem {
        self.s3.as_ref().unwrap().clone()
    }
}
