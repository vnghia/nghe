use time::OffsetDateTime;
use typed_path::Utf8TypedPathBuf;

use crate::media::file;

#[derive(Debug, Clone)]
pub struct Entry {
    pub file_type: file::Type,
    pub path: Utf8TypedPathBuf,
    pub size: u64,
    pub last_modified: Option<OffsetDateTime>,
}
