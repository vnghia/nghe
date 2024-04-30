mod common;
mod local;
pub mod path;
mod s3;

pub use common::FsTrait;
pub use local::{scan_local_media_files, LocalFs};
pub use s3::S3Fs;
