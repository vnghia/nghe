use std::time::Duration;

use aws_config::stalled_stream_protection::StalledStreamProtectionConfig;
use aws_config::timeout::TimeoutConfig;
use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::{AggregatedBytes, DateTime};
use aws_sdk_s3::types::Object;
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use concat_string::concat_string;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use time::OffsetDateTime;
use typed_path::Utf8TypedPath;

use super::{entry, path};
use crate::file::{self, audio};
use crate::http::binary;
use crate::{Error, config, error};

#[derive(Debug, Clone)]
pub struct Filesystem {
    client: Client,
    presigned_duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct Path<'b, 'k> {
    pub bucket: &'b str,
    pub key: &'k str,
}

impl Filesystem {
    pub async fn new(tls: &config::filesystem::Tls, s3: &config::filesystem::S3) -> Self {
        let mut http_connector = HttpConnector::new();
        http_connector.enforce_http(false);

        let tls_connector = hyper_tls::native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(tls.accept_invalid_certs)
            .danger_accept_invalid_hostnames(tls.accept_invalid_hostnames)
            .build()
            .expect("Could not build tls connector");

        let config_loader = aws_config::from_env()
            .stalled_stream_protection(if s3.stalled_stream_grace_preriod > 0 {
                StalledStreamProtectionConfig::enabled()
                    .grace_period(Duration::from_secs(s3.stalled_stream_grace_preriod))
                    .build()
            } else {
                StalledStreamProtectionConfig::disabled()
            })
            .http_client(
                HyperClientBuilder::new()
                    .build(HttpsConnector::from((http_connector, tls_connector.into()))),
            );

        let client = Client::from_conf(
            aws_sdk_s3::config::Builder::from(&config_loader.load().await)
                .force_path_style(s3.use_path_style_endpoint)
                .timeout_config(
                    TimeoutConfig::builder()
                        .connect_timeout(Duration::from_secs(s3.connect_timeout))
                        .build(),
                )
                .build(),
        );

        Self { client, presigned_duration: Duration::from_mins(s3.presigned_duration) }
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
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        let Path { bucket, key } = Self::split(path)?;
        self.client.list_objects_v2().bucket(bucket).prefix(key).max_keys(1).send().await?;
        Ok(())
    }

    async fn scan_folder(
        &self,
        sender: entry::Sender,
        prefix: Utf8TypedPath<'_>,
    ) -> Result<(), Error> {
        let Path { bucket, key } = Self::split(prefix)?;
        let prefix = key;
        let mut steam =
            self.client.list_objects_v2().bucket(bucket).prefix(prefix).into_paginator().send();
        let bucket = path::S3::from_str("/").join(bucket);

        while let Some(output) = steam.try_next().await? {
            if let Some(contents) = output.contents {
                for content in contents {
                    if let Some(key) = content.key() {
                        sender.send(bucket.join(key), &content).await?;
                    }
                }
            }
        }

        Ok(())
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

    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error> {
        let path = source.path.to_path();
        let Path { bucket, key } = Self::split(path)?;
        let reader = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .set_range(offset.map(|offset| concat_string!("bytes=", offset.to_string(), "-")))
            .send()
            .await?
            .body
            .into_async_read();
        binary::Response::from_async_read(
            reader,
            &source.property,
            offset,
            #[cfg(test)]
            None,
        )
    }

    async fn transcode_input(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        let Path { bucket, key } = Self::split(path)?;
        Ok(self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(PresigningConfig::expires_in(self.presigned_duration)?)
            .await?
            .uri()
            .to_owned())
    }
}

impl entry::Metadata for Object {
    fn size(&self) -> Result<usize, Error> {
        Ok(self.size().ok_or_else(|| error::Kind::MissingFileSize)?.try_into()?)
    }

    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error> {
        Ok(self
            .last_modified()
            .map(DateTime::as_nanos)
            .map(OffsetDateTime::from_unix_timestamp_nanos)
            .transpose()?)
    }
}
