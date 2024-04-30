mod common;
mod local;
mod s3;

use common::{extension, join, strip_prefix, with_extension};
pub use common::{SongFsInformation, TemporaryFs, TemporaryFsTrait};
pub use local::TemporaryLocalFs;
pub use s3::TemporaryS3Fs;
