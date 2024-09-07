use std::borrow::Cow;

use tokio::sync::mpsc::Sender;
use typed_path::Utf8TypedPath;

use super::Entry;
use crate::Error;

#[derive(Debug, Clone)]
pub enum Impl<'fs> {
    Local(Cow<'fs, super::local::Filesystem>),
    S3(Cow<'fs, super::s3::Filesystem>),
}

impl<'fs> Impl<'fs> {
    pub fn into_owned(self) -> Impl<'static> {
        match self {
            Impl::Local(filesystem) => Impl::Local(Cow::Owned(filesystem.into_owned())),
            Impl::S3(filesystem) => Impl::S3(Cow::Owned(filesystem.into_owned())),
        }
    }
}

pub trait Trait {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error>;
    async fn scan_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: usize,
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

    async fn scan_folder(
        &self,
        path: Utf8TypedPath<'_>,
        minimum_size: usize,
        tx: Sender<Entry>,
    ) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.scan_folder(path, minimum_size, tx).await,
            Impl::S3(filesystem) => filesystem.scan_folder(path, minimum_size, tx).await,
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
    use fake::{Fake, Faker};
    use futures_lite::StreamExt;
    use itertools::Itertools;
    use nghe_api::common::filesystem;
    use rstest::rstest;
    use tokio_stream::wrappers::ReceiverStream;

    use super::Trait as _;
    use crate::filesystem::Entry;
    use crate::media::file;
    use crate::test::filesystem::Trait as _;
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
    #[case(filesystem::Type::S3, "/nghe-bucket", true)]
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

    #[rstest]
    #[case(20, 15, 10)]
    #[case(50, 10, 40)]
    #[tokio::test]
    async fn test_scan_folder(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] filesystem_type: filesystem::Type,
        #[case] minimum_size: usize,
        #[case] n_smaller: usize,
        #[case] n_larger: usize,
    ) {
        let filesystem = mock.to_impl(filesystem_type);
        let prefix = filesystem.prefix().to_path_buf();
        let main_filesystem = filesystem.main().into_owned();

        let mut entries = vec![];

        for _ in 0..n_smaller {
            let content = fake::vec![u8; 0..minimum_size];
            let path = prefix
                .join(Faker.fake::<String>())
                .with_extension(Faker.fake::<file::Type>().as_ref());
            filesystem.write(path.to_path(), &content).await;
        }

        for _ in 0..n_larger {
            let content = fake::vec![u8; (minimum_size + 1)..(2 * minimum_size)];
            let path = prefix
                .join(Faker.fake::<String>())
                .with_extension(Faker.fake::<file::Type>().as_ref());
            filesystem.write(path.to_path(), &content).await;
            entries.push(Entry {
                file_type: crate::media::file::Type::Flac,
                path,
                size: content.len(),
                last_modified: None,
            });
        }

        let (tx, rx) = tokio::sync::mpsc::channel(mock.config.filesystem.scan.channel_size);
        let scan_handle = tokio::spawn(async move {
            main_filesystem.scan_folder(prefix.to_path(), minimum_size, tx).await.unwrap();
        });
        let scanned_entries: Vec<_> = ReceiverStream::new(rx).collect().await;
        scan_handle.await.unwrap();

        assert_eq!(scanned_entries.len(), n_larger);
        assert_eq!(
            scanned_entries.into_iter().sorted().collect_vec(),
            entries.into_iter().sorted().collect_vec()
        );
    }
}
