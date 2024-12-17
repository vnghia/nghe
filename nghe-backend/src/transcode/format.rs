use std::num::NonZeroU64;

use axum_extra::headers::{CacheControl, ETag};
use nghe_api::common::format;

use crate::Error;
use crate::http::binary;

impl binary::property::Trait for format::Transcode {
    const SEEKABLE: bool = false;

    fn mime(&self) -> &'static str {
        format::Trait::mime(self)
    }

    fn size(&self) -> Option<NonZeroU64> {
        None
    }

    fn etag(&self) -> Result<Option<ETag>, Error> {
        Ok(None)
    }

    fn cache_control() -> CacheControl {
        CacheControl::new().with_no_cache()
    }
}
