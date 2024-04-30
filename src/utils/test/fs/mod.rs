mod common;
mod local;

use common::{extension, join, strip_prefix, with_extension};
pub use common::{SongFsInformation, TemporaryFs, TemporaryFsTrait};
pub use local::TemporaryLocalFs;
