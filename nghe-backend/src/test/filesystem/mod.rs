mod common;
mod local;
mod s3;

pub use common::{MockImpl, MockTrait};
use nghe_api::music_folder::FilesystemType;

use crate::app::state;
use crate::config;

#[derive(Debug)]
pub struct Mock {
    state: state::Filesystem,
    local: local::Mock,
    s3: s3::Mock,
}

impl Mock {
    pub async fn new() -> Self {
        let state = state::Filesystem::new(
            &config::filesystem::Tls::default(),
            &config::filesystem::S3::default(),
        )
        .await;
        let local = local::Mock::new(state.local());
        let s3 = s3::Mock::new(state.s3()).await;

        Self { state, local, s3 }
    }

    pub fn state(&self) -> &state::Filesystem {
        &self.state
    }

    pub fn to_impl(&self, filesystem_type: FilesystemType) -> MockImpl<'_> {
        match filesystem_type {
            FilesystemType::Local => (&self.local).into(),
            FilesystemType::S3 => (&self.s3).into(),
        }
    }
}
