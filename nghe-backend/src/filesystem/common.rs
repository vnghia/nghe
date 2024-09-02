use enum_dispatch::enum_dispatch;
use typed_path::Utf8TypedPath;

use crate::Error;

#[enum_dispatch]
#[derive(Debug)]
pub enum Impl<'fs> {
    Local(&'fs super::local::Filesystem),
    S3(&'fs super::s3::Filesystem),
}

#[enum_dispatch(Impl)]
#[cfg_attr(test, enum_dispatch(MockImpl))]
pub trait Trait {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error>;
}
