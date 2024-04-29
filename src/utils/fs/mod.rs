mod common;
mod local;
pub mod path;

pub use common::FsTrait;
pub use local::{scan_local_media_files, LocalFs};
