mod common;
mod local;
mod s3;

pub use common::{Impl, Trait};
use nghe_api::common::filesystem;

use crate::app::state;

#[derive(Debug)]
pub struct Mock {
    state: state::Filesystem,
    local: local::Mock,
    s3: s3::Mock,
}

impl Mock {
    pub async fn new(config: &super::Config) -> Self {
        let state = state::Filesystem::new(&config.filesystem.tls, &config.filesystem.s3).await;
        let local = local::Mock::new(state.local());
        let s3 = s3::Mock::new(state.s3()).await;

        Self { state, local, s3 }
    }

    pub fn state(&self) -> &state::Filesystem {
        &self.state
    }

    pub fn to_impl(&self, filesystem_type: filesystem::Type) -> Impl<'_> {
        match filesystem_type {
            filesystem::Type::Local => Impl::Local(&self.local),
            filesystem::Type::S3 => Impl::S3(&self.s3),
        }
    }
}
