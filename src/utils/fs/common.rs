use std::fmt::Debug;

use anyhow::Result;
use typed_path::{Utf8Encoding, Utf8Path};

pub trait FsTrait: Debug + Clone
where
    for<'enc> Self::E: Utf8Encoding<'enc> + Debug,
{
    type E;

    async fn read<P: AsRef<Utf8Path<Self::E>>>(&self, path: P) -> Result<Vec<u8>>;
    async fn read_to_string<P: AsRef<Utf8Path<Self::E>>>(&self, path: P) -> Result<String>;
}
