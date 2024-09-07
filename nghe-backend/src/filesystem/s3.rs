use std::time::Duration;

use aws_config::stalled_stream_protection::StalledStreamProtectionConfig;
use aws_config::timeout::TimeoutConfig;
use aws_sdk_s3::primitives::{AggregatedBytes, DateTime};
use aws_sdk_s3::Client;
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use concat_string::concat_string;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use time::OffsetDateTime;
use tokio::sync::mpsc::Sender;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use super::Entry;
use crate::media::file;
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
            && let Some((bucket, key)) = path.split_once('/')
        {
            Ok(Path { bucket, key })
        } else {
            Err(color_eyre::eyre::eyre!(
                "S3 path must be an unix path and have at least two components"
            )
            .into())
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
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<Entry>,
    ) -> Result<(), Error> {
        let Path { bucket, key } = Self::split(path)?;
        let mut steam =
            self.client.list_objects_v2().bucket(bucket).prefix(key).into_paginator().send();

        while let Some(output) = steam.try_next().await.map_err(color_eyre::Report::new)? {
            if let Some(contents) = output.contents {
                for content in contents {
                    if let Some(key) = content.key()
                        && let Some(size) = content.size()
                        && let Ok(size) = size.try_into()
                        && size > minimum_size
                    {
                        let path =
                            Utf8TypedPathBuf::from_unix(concat_string!("/", bucket, "/", key));
                        if let Some(extension) = path.extension()
                            && let Ok(file_type) = file::Type::try_from(extension)
                        {
                            tx.send(Entry {
                                file_type,
                                path,
                                size,
                                last_modified: content
                                    .last_modified()
                                    .map(DateTime::as_nanos)
                                    .map(OffsetDateTime::from_unix_timestamp_nanos)
                                    .transpose()
                                    .map_err(color_eyre::Report::new)?,
                            })
                            .await?;
                        }
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
