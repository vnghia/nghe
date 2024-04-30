use anyhow::Result;
use aws_sdk_s3::primitives::ByteStream;
use concat_string::concat_string;

use super::{extension, join, strip_prefix, with_extension, TemporaryFsTrait};
use crate::utils::fs::{FsTrait, S3Fs};

pub struct TemporaryS3Fs {
    fs: S3Fs,
}

impl TemporaryS3Fs {
    pub async fn new() -> Self {
        let endpoint_url = std::env::var("AWS_ENDPOINT_URL").ok();
        let use_path_style_endpoint = std::env::var("AWS_USE_PATH_STYLE_ENDPOINT").is_ok();
        Self { fs: S3Fs::new(endpoint_url, use_path_style_endpoint).await }
    }
}

#[async_trait::async_trait]
impl TemporaryFsTrait for TemporaryS3Fs {
    fn prefix(&self) -> &str {
        "/"
    }

    fn join(&self, base: &str, path: &str) -> String {
        join::<S3Fs>(base, path)
    }

    fn strip_prefix<'a>(&self, path: &'a str, base: &str) -> &'a str {
        strip_prefix::<S3Fs>(path, base)
    }

    fn extension<'a>(&self, path: &'a str) -> &'a str {
        extension::<S3Fs>(path)
    }

    fn with_extension(&self, path: &str, extension: &str) -> String {
        with_extension::<S3Fs>(path, extension)
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        self.fs.read(path).await
    }

    async fn read_to_string(&self, path: &str) -> Result<String> {
        self.fs.read_to_string(path).await
    }

    async fn mkdir(&self, path: &str) {
        // only create bucket if needed
        let path = concat_string!(path.trim_end_matches('/'), "/");
        let (bucket, _) = S3Fs::split(&path).unwrap();
        if self.fs.client.head_bucket().bucket(bucket).send().await.is_err() {
            self.fs.client.create_bucket().bucket(bucket).send().await.unwrap();
        }
    }

    async fn write(&self, path: &str, data: &[u8]) {
        let (bucket, key) = S3Fs::split(path).unwrap();
        self.fs
            .client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await
            .unwrap();
    }

    async fn remove(&self, path: &str) {
        let (bucket, key) = S3Fs::split(path).unwrap();
        self.fs.client.delete_object().bucket(bucket).key(key).send().await.unwrap();
    }
}
