use std::fmt::Debug;

use anyhow::Result;
use aws_sdk_s3::primitives::AggregatedBytes;
use aws_sdk_s3::Client;
use concat_string::concat_string;
use flume::Sender;
use tokio::task::JoinHandle;
use tracing::instrument;
use typed_path::{Utf8Path, Utf8PathBuf, Utf8UnixEncoding};

use super::FsTrait;
use crate::utils::path::{PathInfo, PathMetadata};
use crate::utils::song::file_type::SUPPORTED_EXTENSIONS;
use crate::OSError;

#[derive(Debug, Clone)]
pub struct S3Fs {
    pub client: Client,
}

impl S3Fs {
    pub async fn new(endpoint_url: Option<String>, use_path_style_endpoint: bool) -> Self {
        let mut config_loader = aws_config::from_env();
        if let Some(endpoint_url) = endpoint_url {
            config_loader = config_loader.endpoint_url(endpoint_url)
        }

        let client = Client::from_conf(
            aws_sdk_s3::config::Builder::from(&config_loader.load().await)
                .force_path_style(use_path_style_endpoint)
                .build(),
        );
        Self { client }
    }

    pub fn split(path: &str) -> Result<(&str, &str)> {
        let path = path
            .strip_prefix('/')
            .ok_or_else(|| OSError::InvalidParameter("s3 path must start with '/'".into()))?;
        // In the latter case, the path only contains the name of the bucket.
        if let Some(result) = path.split_once('/') { Ok(result) } else { Ok((path, "")) }
    }

    pub fn absolutize<P: AsRef<str>>(path: P) -> Utf8PathBuf<Utf8UnixEncoding> {
        concat_string!("/", path).into()
    }

    #[instrument(skip(tx))]
    async fn list_object<P: AsRef<Utf8Path<<Self as FsTrait>::E>> + Debug + Send + Sync>(
        prefix: P,
        tx: Sender<PathInfo<Self>>,
        client: Client,
    ) {
        tracing::info!("start listing object");

        match try {
            let (bucket, prefix) = Self::split(prefix.as_ref().as_str())?;
            let mut stream =
                client.list_objects_v2().bucket(bucket).prefix(prefix).into_paginator().send();
            let bucket = Self::absolutize(bucket);

            while let Some(output) = stream.try_next().await? {
                if let Some(contents) = output.contents {
                    for content in contents {
                        let path = bucket.join(
                            content.key.ok_or_else(|| OSError::NotFound("Object key".into()))?,
                        );
                        if let Some(extension) = path.extension()
                            && SUPPORTED_EXTENSIONS.contains_key(extension)
                        {
                            tx.send(PathInfo {
                                path,
                                metadata: PathMetadata {
                                    size: content
                                        .size
                                        .ok_or_else(|| OSError::NotFound("Object size".into()))?
                                        as _,
                                },
                            })?
                        }
                    }
                }
            }
        } {
            Ok(()) => (),
            Err::<_, anyhow::Error>(e) => {
                tracing::error!(listing_object = ?e);
                return;
            }
        };

        tracing::info!("finish listing object");
    }
}

#[async_trait::async_trait]
impl FsTrait for S3Fs {
    type E = Utf8UnixEncoding;

    async fn read<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(&self, path: P) -> Result<Vec<u8>> {
        let (bucket, key) = Self::split(path.as_ref().as_str())?;
        self.client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?
            .body
            .collect()
            .await
            .map(AggregatedBytes::to_vec)
            .map_err(anyhow::Error::from)
    }

    async fn read_to_string<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<String> {
        String::from_utf8(self.read(path).await?).map_err(anyhow::Error::from)
    }

    fn scan_songs<P: AsRef<Utf8Path<Self::E>> + Debug + Send + Sync + 'static>(
        &self,
        prefix: P,
        tx: Sender<PathInfo<Self>>,
    ) -> JoinHandle<()> {
        let span = tracing::Span::current();
        let client = self.client.clone();
        tokio::task::spawn(async move {
            let _enter = span.enter();
            Self::list_object(prefix, tx, client).await
        })
    }
}

#[cfg(test)]
mod tests {
    use futures::{stream, StreamExt};
    use itertools::Itertools;

    use super::*;
    use crate::utils::test::Infra;

    const FS_INDEX: usize = 1;

    async fn wrap_scan_media_file(infra: &Infra) -> Vec<PathInfo<S3Fs>> {
        let (tx, rx) = flume::bounded(100);
        let scan_task = infra.fs.s3().scan_songs(infra.fs.prefix(FS_INDEX).to_string(), tx);
        let mut result = vec![];
        while let Ok(r) = rx.recv_async().await {
            result.push(r);
        }
        scan_task.await.unwrap();
        result
    }

    #[tokio::test]
    async fn test_scan_media_files_no_filter() {
        let infra = Infra::new().await;
        let fs = &infra.fs;

        let media_paths = stream::iter(infra.fs.mkrelpaths(
            FS_INDEX,
            50,
            3,
            &SUPPORTED_EXTENSIONS.keys().collect_vec(),
        ))
        .then(move |path| async move { fs.mkfile(FS_INDEX, &path).await })
        .collect::<Vec<_>>()
        .await;

        let scanned_results = wrap_scan_media_file(&infra).await;
        let scanned_paths =
            scanned_results.iter().cloned().map(|result| result.path.to_string()).collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_extension() {
        let infra = Infra::new().await;
        let fs = &infra.fs;

        let media_paths = stream::iter(
            infra.fs.mkrelpaths(
                FS_INDEX,
                50,
                3,
                &[SUPPORTED_EXTENSIONS.keys().copied().collect_vec().as_slice(), &["txt", "rs"]]
                    .concat(),
            ),
        )
        .filter_map(move |path| async move {
            let path = fs.mkfile(FS_INDEX, &path).await;
            let extension = fs.extension(FS_INDEX, &path);
            if SUPPORTED_EXTENSIONS.contains_key(extension) { Some(path) } else { None }
        })
        .collect::<Vec<_>>()
        .await;

        let scanned_paths = wrap_scan_media_file(&infra)
            .await
            .into_iter()
            .map(|result| result.path.to_string())
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }

    #[tokio::test]
    async fn test_scan_media_files_filter_dir() {
        let infra = Infra::new().await;
        let fs = &infra.fs;

        let media_paths = stream::iter(fs.mkrelpaths(
            FS_INDEX,
            50,
            3,
            &SUPPORTED_EXTENSIONS.keys().collect_vec(),
        ))
        .filter_map(move |path| async move {
            if rand::random::<bool>() {
                Some(fs.mkfile(FS_INDEX, &path).await)
            } else {
                fs.mkdir(FS_INDEX, &path).await;
                None
            }
        })
        .collect::<Vec<_>>()
        .await;

        let scanned_paths = wrap_scan_media_file(&infra)
            .await
            .into_iter()
            .map(|result| result.path.to_string())
            .collect_vec();

        assert_eq!(
            media_paths.into_iter().sorted().collect_vec(),
            scanned_paths.into_iter().sorted().collect_vec()
        );
    }
}
