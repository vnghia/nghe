mod common;
mod local;
mod s3;

pub use common::{Impl, Trait};
use nghe_api::common::filesystem;

use crate::filesystem::Filesystem;

#[derive(Debug)]
pub struct Mock {
    state: Filesystem,
    local: local::Mock,
    s3: s3::Mock,
}

impl Mock {
    pub async fn new(prefix: Option<&str>, config: &super::Config) -> Self {
        let state = Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;
        let local = local::Mock::new(state.local());
        let s3 = s3::Mock::new(prefix, state.s3()).await;

        Self { state, local, s3 }
    }

    pub fn state(&self) -> &Filesystem {
        &self.state
    }

    pub fn to_impl(&self, filesystem_type: filesystem::Type) -> Impl<'_> {
        match filesystem_type {
            filesystem::Type::Local => Impl::Local(&self.local),
            filesystem::Type::S3 => Impl::S3(&self.s3),
        }
    }
}
