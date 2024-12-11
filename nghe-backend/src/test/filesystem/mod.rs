mod common;
mod local;
mod s3;

pub use common::{Impl, Trait};
use nghe_api::common::filesystem;
use typed_path::Utf8PlatformPathBuf;

use crate::filesystem::Filesystem;

pub struct Mock {
    filesystem: Filesystem,
    local: local::Mock,
    s3: s3::Mock,
}

impl Mock {
    pub async fn new(prefix: Option<&str>, config: &super::Config) -> Self {
        let filesystem = Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;
        let local = local::Mock::new(filesystem.local());
        let s3 = s3::Mock::new(prefix, filesystem.s3()).await;

        Self { filesystem, local, s3 }
    }

    pub fn filesystem(&self) -> &Filesystem {
        &self.filesystem
    }

    pub fn to_impl(&self, ty: filesystem::Type) -> Impl<'_> {
        match ty {
            filesystem::Type::Local => Impl::Local(&self.local),
            filesystem::Type::S3 => Impl::S3(&self.s3),
        }
    }

    pub fn prefix(&self) -> Utf8PlatformPathBuf {
        crate::filesystem::local::Filesystem::to_platform(self.local.prefix()).unwrap()
    }
}
