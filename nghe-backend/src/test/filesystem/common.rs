use enum_dispatch::enum_dispatch;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::filesystem::Trait;
use crate::Error;

#[enum_dispatch]
#[derive(Debug)]
pub enum MockImpl<'fs> {
    Local(&'fs super::local::Mock),
}

#[enum_dispatch(MockImpl)]
pub trait MockTrait: Trait {
    fn prefix(&self) -> Utf8TypedPath<'_>;

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf;
    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]);
    async fn delete(&self, path: Utf8TypedPath<'_>);
}
