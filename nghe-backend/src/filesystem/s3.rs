use std::time::Duration;

use aws_config::stalled_stream_protection::StalledStreamProtectionConfig;
use aws_config::timeout::TimeoutConfig;
use aws_sdk_s3::primitives::{AggregatedBytes, DateTime};
use aws_sdk_s3::types::Object;
use aws_sdk_s3::Client;
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use time::OffsetDateTime;
use typed_path::Utf8TypedPath;

use super::{entry, path};
use crate::{config, Error};

#[derive(Debug, Clone)]
pub struct Filesystem {
    client: Client,
    presigned_url_duration: u64,
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

        Self { client, presigned_url_duration: s3.presigned_url_duration }
    }

    pub fn split<'b, 'k, 'p: 'b + 'k>(
        path: impl Into<Utf8TypedPath<'p>>,
    ) -> Result<Path<'b, 'k>, Error> {
        let path = path.into();
        if let Utf8TypedPath::Unix(path) = path
            && path.is_absolute()
            && let Some(path) = path.as_str().strip_prefix('/')
        {
            if let Some((bucket, key)) = path.split_once('/') {
                Ok(Path { bucket, key })
            } else {
                Ok(Path { bucket: path, key: "" })
            }
        } else {
            Err(Error::FilesystemS3InvalidPath(path.to_string()))
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
        self.client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(key)
            .max_keys(1)
            .send()
            .await
            .map_err(color_eyre::Report::new)?;
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

        while let Some(output) = steam.try_next().await.map_err(color_eyre::Report::new)? {
            if let Some(contents) = output.contents {
                for content in contents {
                    if let Some(path) = content.key() {
                        sender.send(prefix, path::S3::from_str(path), &content).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        let Path { bucket, key } = Self::split(path)?;
        self.client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(color_eyre::Report::new)?
            .body
            .collect()
            .await
            .map(AggregatedBytes::to_vec)
            .map_err(Error::from)
    }
}

impl entry::Metadata for Object {
    fn size(&self) -> Result<usize, Error> {
        Ok(self.size().ok_or_else(|| Error::FilesystemS3MissingObjectSize)?.try_into()?)
    }

    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error> {
        Ok(self
            .last_modified()
            .map(DateTime::as_nanos)
            .map(OffsetDateTime::from_unix_timestamp_nanos)
            .transpose()
            .map_err(color_eyre::Report::new)?)
    }
}
