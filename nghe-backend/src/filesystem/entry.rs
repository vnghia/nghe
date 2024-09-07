use time::OffsetDateTime;
use typed_path::Utf8TypedPathBuf;

use crate::media::file;

#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative))]
#[cfg_attr(test, derivative(PartialEq, Eq, PartialOrd, Ord))]
pub struct Entry {
    pub file_type: file::Type,
    pub path: Utf8TypedPathBuf,
    pub size: usize,
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    #[cfg_attr(test, derivative(PartialOrd = "ignore"))]
    #[cfg_attr(test, derivative(Ord = "ignore"))]
    pub last_modified: Option<OffsetDateTime>,
}
