use std::fmt::Debug;

use anyhow::Result;
use flume::Sender;
use tokio::task::JoinHandle;
use typed_path::{Utf8Encoding, Utf8Path};

use crate::utils::path::PathInfo;

#[async_trait::async_trait]
pub trait FsTrait: Debug + Clone + Send + Sync
where
    for<'enc> Self::E: Utf8Encoding<'enc> + Debug + Send + Sync,
{
    type E;

    async fn read<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(&self, path: P) -> Result<Vec<u8>>;
    async fn read_to_string<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<String>;

    fn scan_songs<P: AsRef<Utf8Path<Self::E>> + Debug + Send + Sync + 'static>(
        &self,
        prefix: P,
        tx: Sender<PathInfo<Self>>,
    ) -> JoinHandle<()>;
}
