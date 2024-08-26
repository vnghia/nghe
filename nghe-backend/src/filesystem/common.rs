use enum_dispatch::enum_dispatch;
use typed_path::Utf8TypedPath;

use crate::Error;

#[enum_dispatch]
#[derive(Debug)]
pub enum Impl<'fs> {
    Local(&'fs super::local::Filesystem),
}

#[enum_dispatch(Impl)]
pub trait Trait {
    async fn check_folder<'a>(&self, path: Utf8TypedPath<'a>) -> Result<Utf8TypedPath<'a>, Error>;
}
