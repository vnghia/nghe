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
    pub async fn new(filesystem: s3::Filesystem) -> Self {
        let bucket = Faker.fake::<String>().to_lowercase();
        filesystem.client().create_bucket().bucket(&bucket).send().await.unwrap();
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

    async fn list_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<filesystem::Entry>,
    ) -> Result<(), Error> {
        self.filesystem.list_folder(path, minimum_size, tx).await
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        self.filesystem.read(path).await
    }
}

impl super::Trait for Mock {
    fn prefix(&self) -> Utf8TypedPath<'_> {
        self.bucket.to_path()
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
