use std::borrow::Cow;

use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::create_bucket::CreateBucketError;
use aws_sdk_s3::Client;
use concat_string::concat_string;
use fake::{Fake, Faker};
use tokio::sync::mpsc::Sender;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::filesystem::{self, s3};
use crate::Error;

#[derive(Debug)]
pub struct Mock {
    bucket: Utf8TypedPathBuf,
    filesystem: s3::Filesystem,
}

impl Mock {
    pub async fn new(prefix: Option<&str>, filesystem: s3::Filesystem) -> Self {
        let bucket =
            prefix.map_or_else(|| Faker.fake::<String>().to_lowercase().into(), Cow::Borrowed);

        let result = filesystem.client().create_bucket().bucket(bucket.clone()).send().await;
        if result.is_err() {
            if let Err(SdkError::ServiceError(err)) =
                filesystem.client().create_bucket().bucket(bucket.clone()).send().await
                && let CreateBucketError::BucketAlreadyOwnedByYou(_) = err.into_err()
            {
            } else {
                panic!("Could not create bucket {bucket}")
            }
        }

        let bucket = Utf8TypedPathBuf::from_unix(concat_string!("/", bucket));
        assert!(bucket.is_absolute());
        Self { bucket, filesystem }
    }

    pub fn client(&self) -> &Client {
        self.filesystem.client()
    }
}

impl filesystem::Trait for Mock {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        self.filesystem.check_folder(path).await
    }

    async fn scan_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<filesystem::Entry>,
    ) -> Result<(), Error> {
        self.filesystem.scan_folder(path, minimum_size, tx).await
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        self.filesystem.read(path).await
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
        let s3::Path { bucket, key } = s3::Filesystem::split(path).unwrap();
        self.client()
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(aws_sdk_s3::primitives::ByteStream::from(data.to_vec()))
            .send()
            .await
            .unwrap();
    }

    async fn delete(&self, path: Utf8TypedPath<'_>) {
        let s3::Path { bucket, key } = s3::Filesystem::split(path).unwrap();
        self.client().delete_object().bucket(bucket).key(key).send().await.unwrap();
    }
}
