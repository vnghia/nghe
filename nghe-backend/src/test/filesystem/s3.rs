use std::borrow::Cow;

use concat_string::concat_string;
use fake::{Fake, Faker};
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::Error;
use crate::file::{self, audio};
use crate::filesystem::{self, path, s3};
use crate::http::binary;

#[derive(Debug)]
pub struct Mock {
    bucket: Utf8TypedPathBuf,
    filesystem: s3::Filesystem,
}

impl Mock {
    pub async fn new(prefix: Option<&str>, filesystem: s3::Filesystem) -> Self {
        let bucket =
            prefix.map_or_else(|| Faker.fake::<String>().to_lowercase().into(), Cow::Borrowed);

        if filesystem.client().buckets().head(bucket.clone()).send().await.is_err() {
            filesystem
                .client()
                .buckets()
                .create(bucket.clone())
                .send()
                .await
                .expect("Could not create bucket {bucket}");
        }

        let bucket = path::S3::from_string(concat_string!("/", bucket));
        assert!(bucket.is_absolute());
        Self { bucket, filesystem }
    }

    pub fn client(&self) -> &::s3::Client {
        self.filesystem.client()
    }
}

impl filesystem::Trait for Mock {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        self.filesystem.check_folder(path).await
    }

    async fn scan_folder(
        &self,
        sender: filesystem::entry::Sender,
        prefix: Utf8TypedPath<'_>,
    ) -> Result<(), Error> {
        self.filesystem.scan_folder(sender, prefix).await
    }

    async fn exists(&self, path: Utf8TypedPath<'_>) -> Result<bool, Error> {
        self.filesystem.exists(path).await
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        self.filesystem.read(path).await
    }

    async fn read_to_string(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        self.filesystem.read_to_string(path).await
    }

    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error> {
        self.filesystem.read_to_binary(source, offset).await
    }

    async fn transcode_input(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        self.filesystem.transcode_input(path).await
    }
}

impl super::Trait for Mock {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        self.bucket.to_path()
    }

    fn main(&self) -> filesystem::Impl<'_> {
        filesystem::Impl::S3(Cow::Borrowed(&self.filesystem))
    }

    async fn create_dir(&self, path: Utf8TypedPath<'_>) -> Utf8TypedPathBuf {
        self.prefix().join(path)
    }

    async fn write(&self, path: Utf8TypedPath<'_>, data: &[u8]) {
        let path = self.absolutize(path);
        let s3::Path { bucket, key } = s3::Filesystem::split(path.to_path()).unwrap();
        self.client().objects().put(bucket, key).body_bytes(data.to_owned()).send().await.unwrap();
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        let path = self.absolutize(path);
        let s3::Path { bucket, key } = s3::Filesystem::split(path.to_path()).unwrap();
        self.client().objects().delete(bucket, key).send().await.unwrap();
    }
}
