use std::any::Any;

use anyhow::Result;
use aws_sdk_s3::primitives::ByteStream;
use typed_path::{Utf8PathBuf, Utf8UnixEncoding};

use super::{extension, join, strip_prefix, with_extension, TemporaryFs, TemporaryFsTrait};
use crate::utils::fs::{FsTrait, S3Fs};

pub struct TemporaryS3Fs {
    fs: S3Fs,
    bucket: Utf8PathBuf<Utf8UnixEncoding>,
}

impl TemporaryS3Fs {
    pub async fn new() -> Self {
        let fs = S3Fs::new(Default::default()).await;

        let bucket = TemporaryFs::fake_fs_name();
        fs.client.create_bucket().bucket(&bucket).send().await.unwrap();
        let bucket = S3Fs::absolutize(bucket);
        assert!(bucket.is_absolute());

        Self { fs, bucket }
    }
}

#[async_trait::async_trait]
impl TemporaryFsTrait for TemporaryS3Fs {
    fn prefix(&self) -> &str {
        self.bucket.as_str()
    }

    fn fs(&self) -> &dyn Any {
        &self.fs
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

    async fn mkdir(&self, _: &str) {}

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
