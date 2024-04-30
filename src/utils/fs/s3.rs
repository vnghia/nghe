use anyhow::Result;
use aws_sdk_s3::primitives::AggregatedBytes;
use aws_sdk_s3::types::Object;
use aws_sdk_s3::Client;
use typed_path::{Utf8Path, Utf8UnixEncoding};

use super::FsTrait;
use crate::utils::path::PathMetadata;
use crate::OSError;

#[derive(Debug, Clone)]
pub struct S3Fs {
    pub client: Client,
}

impl From<Object> for PathMetadata {
    fn from(value: Object) -> Self {
        Self { size: value.size.expect("s3 object size is missing") as _ }
    }
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
        path.strip_prefix('/')
            .ok_or_else(|| OSError::InvalidParameter("s3 path must start with '/'".into()))?
            .split_once('/')
            .ok_or_else(|| {
                anyhow::anyhow!(OSError::InvalidParameter(
                    "s3 path must has form /bucket/key".into()
                ))
            })
    }
}

impl FsTrait for S3Fs {
    type E = Utf8UnixEncoding;

    async fn read<P: AsRef<Utf8Path<Self::E>>>(&self, path: P) -> Result<Vec<u8>> {
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

    async fn read_to_string<P: AsRef<Utf8Path<Self::E>>>(&self, path: P) -> Result<String> {
        String::from_utf8(self.read(path).await?).map_err(anyhow::Error::from)
    }
}
