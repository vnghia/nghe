use aws_config::stalled_stream_protection::StalledStreamProtectionConfig;
use aws_config::timeout::TimeoutConfig;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::AggregatedBytes;
use aws_sdk_s3::types::Object;
use aws_smithy_types_convert::stream::PaginationStreamExt;
use concat_string::concat_string;
use educe::Educe;
use futures_lite::{Stream, StreamExt, stream};
use time::OffsetDateTime;
use typed_path::Utf8TypedPath;

use super::entry::Entry;
use super::{entry, path};
use crate::{Error, config, error};
#[derive(Clone, Educe)]
#[educe(Debug)]
pub struct Filesystem {
    #[educe(Debug(ignore))]
    client: Client,
}

#[derive(Debug, Clone, Copy)]
pub struct Path<'b, 'k> {
    pub bucket: &'b str,
    pub key: &'k str,
}

impl Filesystem {
    pub async fn new(_tls: &config::filesystem::Tls, s3: &config::filesystem::S3) -> Self {
        let config = aws_config::from_env()
            .stalled_stream_protection(if s3.stalled_stream_grace_preriod > 0 {
                StalledStreamProtectionConfig::enabled()
                    .grace_period(std::time::Duration::from_secs(s3.stalled_stream_grace_preriod))
                    .build()
            } else {
                StalledStreamProtectionConfig::disabled()
            })
            .load()
            .await;
        let client = Client::from_conf(
            aws_sdk_s3::config::Builder::from(&config)
                .force_path_style(s3.use_path_style_endpoint)
                .timeout_config(
                    TimeoutConfig::builder()
                        .connect_timeout(std::time::Duration::from_secs(s3.timeout))
                        .build(),
                )
                .build(),
        );

        Self { client }
    }

    pub fn split<'b, 'k, 'p: 'b + 'k>(
        path: impl Into<Utf8TypedPath<'p>>,
    ) -> Result<Path<'b, 'k>, Error> {
        let path = path.into();
        if let Utf8TypedPath::Unix(path) = path {
            if path.is_absolute()
                && let Some(path) = path.as_str().strip_prefix('/')
            {
                if let Some((bucket, key)) = path.split_once('/') {
                    Ok(Path { bucket, key })
                } else {
                    Ok(Path { bucket: path, key: "" })
                }
            } else {
                error::Kind::InvalidAbsolutePath(path.to_typed_path_buf()).into()
            }
        } else {
            error::Kind::InvalidTypedPathPlatform(path.to_path_buf()).into()
        }
    }

    #[cfg(test)]
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl super::Trait for Filesystem {
    type EntryStream = impl Stream<Item = Result<Entry, Error>>;

    type AsyncRead = impl tokio::io::AsyncBufRead + Unpin;
    type Read = tokio_util::io::SyncIoBridge<impl tokio::io::AsyncBufRead + Unpin>;

    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        let Path { bucket, key } = Self::split(path)?;
        self.client.list_objects_v2().bucket(bucket).prefix(key).max_keys(1).send().await?;
        Ok(())
    }

    fn scan_folder(
        &self,
        prefix: Utf8TypedPath<'_>,
        minimum_size: usize,
    ) -> Result<Self::EntryStream, Error> {
        let Path { bucket, key } = Self::split(prefix)?;
        let root = path::S3::from_str("/").join(bucket);
        Ok(self
            .client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(key)
            .into_paginator()
            .send()
            .into_stream_03x()
            .flat_map(move |result| {
                stream::iter(match result {
                    Ok(output) => output
                        .contents
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|content| {
                            if let Some(key) = content.key() {
                                Entry::try_new(minimum_size, root.join(key), &content)
                            } else {
                                Ok(None)
                            }
                            .transpose()
                        })
                        .collect::<Vec<_>>(),
                    Err(error) => vec![Err(error.into())],
                })
            }))
    }

    async fn exists(&self, path: Utf8TypedPath<'_>) -> Result<bool, Error> {
        let Path { bucket, key } = Self::split(path)?;
        let result = self.client.head_object().bucket(bucket).key(key).send().await;
        if let Err(error) = result {
            let error: Error = error.into();
            if error.status_code == axum::http::StatusCode::NOT_FOUND {
                Ok(false)
            } else {
                Err(error)
            }
        } else {
            Ok(true)
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        let Path { bucket, key } = Self::split(path)?;
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
            .map_err(Error::from)
    }

    async fn read_to_string(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        String::from_utf8(self.read(path).await?).map_err(Error::from)
    }

    async fn read_to_async_read(
        &self,
        path: Utf8TypedPath<'_>,
        offset: Option<u64>,
    ) -> Result<Self::AsyncRead, Error> {
        let Path { bucket, key } = Self::split(path)?;
        Ok(self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .set_range(offset.map(|offset| concat_string!("bytes=", offset.to_string(), "-")))
            .send()
            .await?
            .body
            .into_async_read())
    }

    async fn read_to_read(
        &self,
        path: Utf8TypedPath<'_>,
        offset: Option<u64>,
    ) -> Result<Self::Read, Error> {
        Ok(tokio_util::io::SyncIoBridge::new(self.read_to_async_read(path, offset).await?))
    }
}

impl entry::Metadata for Object {
    fn size(&self) -> Result<usize, Error> {
        Ok(self.size().ok_or_else(|| error::Kind::MissingFileSize)?.try_into()?)
    }

    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error> {
        Ok(self
            .last_modified()
            .map(aws_smithy_types_convert::date_time::DateTimeExt::to_time)
            .transpose()?)
    }
}
