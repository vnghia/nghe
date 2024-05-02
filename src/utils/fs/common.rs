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

    fn join<PB: AsRef<Utf8Path<Self::E>>, PP: AsRef<Utf8Path<Self::E>>>(
        base: PB,
        path: PP,
    ) -> Utf8PathBuf<Self::E> {
        base.as_ref().join(path)
    }

    async fn check_folder<'a>(&self, path: &'a Utf8Path<Self::E>) -> Result<&'a str>;

    async fn read<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(&self, path: P) -> Result<Vec<u8>>;
    async fn read_to_string<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<String>;
    async fn read_to_stream<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<StreamResponse>;
    async fn read_to_transcoding_input<P: Into<Utf8PathBuf<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<String>;

    fn scan_songs<P: AsRef<Utf8Path<Self::E>> + Debug + Send + Sync + 'static>(
        &self,
        prefix: P,
        tx: Sender<PathInfo<Self>>,
    ) -> JoinHandle<()>;
}
