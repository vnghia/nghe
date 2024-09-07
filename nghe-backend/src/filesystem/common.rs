use tokio::sync::mpsc::Sender;
use typed_path::Utf8TypedPath;

use super::Entry;
use crate::Error;

#[derive(Debug)]
pub enum Impl<'fs> {
    Local(&'fs super::local::Filesystem),
    S3(&'fs super::s3::Filesystem),
}

pub trait Trait {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error>;
    async fn list_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<Entry>,
    ) -> Result<(), Error>;

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error>;
}

impl<'fs> Trait for Impl<'fs> {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.check_folder(path).await,
            Impl::S3(filesystem) => filesystem.check_folder(path).await,
        }
    }

    async fn list_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: u64,
        tx: Sender<Entry>,
    ) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.list_folder(path, minimum_size, tx).await,
            Impl::S3(filesystem) => filesystem.list_folder(path, minimum_size, tx).await,
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read(path).await,
            Impl::S3(filesystem) => filesystem.read(path).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use nghe_api::common::filesystem;
    use rstest::rstest;

    use super::*;
    use crate::test::{mock, Mock};

    #[rstest]
    #[case(filesystem::Type::Local, "usr/bin", false)]
    #[case(filesystem::Type::Local, "Windows\\Sys64", false)]
    #[cfg_attr(unix, case(filesystem::Type::Local, "/tmp/", true))]
    #[cfg_attr(unix, case(filesystem::Type::Local, "C:\\Windows", false))]
    #[cfg_attr(windows, case(filesystem::Type::Local, "C:\\Windows", true))]
    #[cfg_attr(windows, case(filesystem::Type::Local, "/tmp/", false))]
    #[case(filesystem::Type::S3, "usr/bin", false)]
    #[case(filesystem::Type::S3, "Windows\\Sys64", false)]
    #[case(filesystem::Type::S3, "/tmp", false)]
    #[case(filesystem::Type::S3, "C:\\Windows", false)]
    #[case(filesystem::Type::S3, "/nghe-bucket", false)]
    #[case(filesystem::Type::S3, "/nghe-bucket/test/", true)]
    #[tokio::test]
    async fn test_check_folder(
        #[future(awt)]
        #[with(0, 0, Some("nghe-bucket"))]
        mock: Mock,
        #[case] filesystem_type: filesystem::Type,
        #[case] path: &str,
        #[case] is_ok: bool,
    ) {
        let filesystem = mock.to_impl(filesystem_type);
        assert_eq!(filesystem.check_folder(path.into()).await.is_ok(), is_ok);
    }
}
