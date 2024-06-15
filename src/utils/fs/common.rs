use std::fmt::Debug;

use anyhow::Result;
use flume::Sender;
use tokio::task::JoinHandle;
use typed_path::{Utf8Encoding, Utf8Path, Utf8PathBuf};

use crate::open_subsonic::StreamResponse;
use crate::utils::path::PathInfo;

#[async_trait::async_trait]
pub trait FsTrait: Debug + Clone + Send + Sync
where
    for<'enc> Self::E: Utf8Encoding<'enc> + Debug + Send + Sync,
{
    type E;

    fn join(
        base: impl AsRef<Utf8Path<Self::E>>,
        path: impl AsRef<Utf8Path<Self::E>>,
    ) -> Utf8PathBuf<Self::E> {
        base.as_ref().join(path)
    }

    async fn check_folder<'a>(&self, path: &'a Utf8Path<Self::E>) -> Result<&'a str>;

    async fn read(&self, path: impl AsRef<Utf8Path<Self::E>> + Send + Sync) -> Result<Vec<u8>>;
    async fn read_to_string(
        &self,
        path: impl AsRef<Utf8Path<Self::E>> + Send + Sync,
    ) -> Result<String>;
    async fn read_to_stream(
        &self,
        path: impl AsRef<Utf8Path<Self::E>> + Send + Sync,
        offset: u64,
        size: u64,
    ) -> Result<StreamResponse>;
    async fn read_to_transcoding_input(
        &self,
        path: impl Into<Utf8PathBuf<Self::E>> + Send + Sync,
    ) -> Result<String>;

    fn scan_songs(
        &self,
        prefix: impl AsRef<Utf8Path<Self::E>> + Debug + Send + Sync + 'static,
        tx: Sender<PathInfo<Self>>,
    ) -> JoinHandle<()>;
}
