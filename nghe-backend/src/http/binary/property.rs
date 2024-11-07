use axum_extra::headers::ETag;

use crate::Error;

pub trait Trait: Copy {
    fn mime(&self) -> &'static str;
    fn size(&self) -> u64;
    fn etag(&self) -> Result<Option<ETag>, Error>;
}
