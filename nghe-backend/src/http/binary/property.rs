use axum_extra::headers::{CacheControl, ETag};

use crate::Error;

pub trait Trait: Copy {
    const SEEKABLE: bool;

    fn mime(&self) -> &'static str;
    fn size(&self) -> Option<u64>;
    fn etag(&self) -> Result<Option<ETag>, Error>;

    fn cache_control() -> CacheControl;
}
