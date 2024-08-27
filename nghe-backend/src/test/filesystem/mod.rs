#![allow(clippy::unused_self)]

mod common;
mod local;

pub use common::{MockImpl, MockTrait};
use nghe_api::music_folder::FilesystemType;

use crate::app::state;
use crate::filesystem;

#[derive(Debug)]
pub struct Mock {
    local: local::Mock,

    state: state::Filesystem,
}

impl Mock {
    pub fn new() -> Self {
        let local = local::Mock::new(filesystem::local::Filesystem);

        Self { local, state: state::Filesystem::new() }
    }

    pub fn state(&self) -> &state::Filesystem {
        &self.state
    }

    pub fn to_impl(&self, filesystem_type: FilesystemType) -> MockImpl<'_> {
        match filesystem_type {
            FilesystemType::Local => (&self.local).into(),
            FilesystemType::S3 => todo!(),
        }
    }
}
