mod common;
mod local;
pub mod path;
mod s3;

pub use common::FsTrait;
pub use local::LocalFs;
pub use s3::S3Fs;
