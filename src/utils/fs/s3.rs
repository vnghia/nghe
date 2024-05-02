use std::fmt::Debug;
use std::time::Duration;

use anyhow::Result;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::AggregatedBytes;
use aws_sdk_s3::Client;
use concat_string::concat_string;
use flume::Sender;
use time::OffsetDateTime;
use tokio::task::JoinHandle;
use tracing::instrument;
use typed_path::{Utf8Path, Utf8PathBuf, Utf8UnixEncoding};

use super::FsTrait;
use crate::config::S3Config;
use crate::open_subsonic::StreamResponse;
use crate::utils::path::{PathInfo, PathMetadata};
use crate::utils::song::file_type::SUPPORTED_EXTENSIONS;
use crate::OSError;

#[derive(Debug, Clone)]
pub struct S3Fs {
    pub client: Client,
    pub presigned_url_duration: u64,
}

impl S3Fs {
    pub async fn new(config: S3Config) -> Self {
        let mut config_loader = aws_config::from_env();
        if let Some(endpoint_url) = config.endpoint_url {
            config_loader = config_loader.endpoint_url(endpoint_url)
        }

        let client = Client::from_conf(
            aws_sdk_s3::config::Builder::from(&config_loader.load().await)
                .force_path_style(config.use_path_style_endpoint)
                .build(),
        );
        Self { client, presigned_url_duration: config.presigned_url_duration }
    }

    pub fn unwrap(value: Option<&S3Fs>) -> Result<&S3Fs> {
        value.ok_or_else(|| OSError::InvalidParameter("s3 integration is disabled".into()).into())
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

    #[instrument(skip(tx, client))]
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
                            tx.send_async(PathInfo {
                                path,
                                metadata: PathMetadata {
                                    size: content
                                        .size
                                        .ok_or_else(|| OSError::NotFound("Object size".into()))?
                                        as _,
                                    last_modified: match content.last_modified {
                                        Some(t) => Some(OffsetDateTime::from_unix_timestamp_nanos(
                                            t.as_nanos(),
                                        )?),
                                        None => None,
                                    },
                                },
                            })
                            .await?
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

    async fn check_folder<'a>(&self, path: &'a Utf8Path<Self::E>) -> Result<&'a str> {
        let (bucket, prefix) = Self::split(path.as_str())?;
        self.client.list_objects_v2().bucket(bucket).prefix(prefix).max_keys(1).send().await?;
        Ok(path.as_str())
    }

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

    async fn read_to_stream<P: AsRef<Utf8Path<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<StreamResponse> {
        let path = path.as_ref();
        let (bucket, key) = Self::split(path.as_str())?;
        Ok(StreamResponse::from_async_read(
            path.extension().ok_or_else(|| {
                OSError::InvalidParameter("path does not have an extension".into())
            })?,
            self.client.get_object().bucket(bucket).key(key).send().await?.body.into_async_read(),
        ))
    }

    async fn read_to_transcoding_input<P: Into<Utf8PathBuf<Self::E>> + Send + Sync>(
        &self,
        path: P,
    ) -> Result<String> {
        let path = path.into();
        let (bucket, key) = Self::split(path.as_str())?;
        Ok(self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(PresigningConfig::expires_in(Duration::from_secs(
                self.presigned_url_duration * 60,
            ))?)
            .await?
            .uri()
            .into())
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
    use crate::models::*;
    use crate::utils::test::Infra;

    const FS_TYPE: music_folders::FsType = music_folders::FsType::S3;

    async fn wrap_scan_media_file(infra: &Infra) -> Vec<PathInfo<S3Fs>> {
        let (tx, rx) = flume::bounded(100);
        let scan_task = infra.fs.s3().scan_songs(infra.fs.prefix(FS_TYPE).to_string(), tx);
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
            FS_TYPE,
            50,
            3,
            &SUPPORTED_EXTENSIONS.keys().collect_vec(),
        ))
        .then(move |path| async move { fs.mkfile(FS_TYPE, &path).await })
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
                FS_TYPE,
                50,
                3,
                &[SUPPORTED_EXTENSIONS.keys().copied().collect_vec().as_slice(), &["txt", "rs"]]
                    .concat(),
            ),
        )
        .filter_map(move |path| async move {
            let path = fs.mkfile(FS_TYPE, &path).await;
            let extension = fs.extension(FS_TYPE, &path);
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

        let media_paths =
            stream::iter(fs.mkrelpaths(FS_TYPE, 50, 3, &SUPPORTED_EXTENSIONS.keys().collect_vec()))
                .filter_map(move |path| async move {
                    if rand::random::<bool>() {
                        Some(fs.mkfile(FS_TYPE, &path).await)
                    } else {
                        fs.mkdir(FS_TYPE, &path).await;
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
