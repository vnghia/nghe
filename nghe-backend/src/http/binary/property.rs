use std::fmt::Debug;
use std::num::NonZeroU64;

use axum_extra::headers::{CacheControl, ETag};

use crate::Error;

pub trait Trait: Debug + Copy {
    const SEEKABLE: bool;

    fn mime(&self) -> &'static str;
    fn size(&self) -> Option<NonZeroU64>;
    fn etag(&self) -> Result<Option<ETag>, Error>;

    fn cache_control() -> CacheControl;
}
