use std::borrow::Cow;
use std::ffi::CString;

use nghe_api::common::filesystem;
use o2o::o2o;
use typed_path::Utf8TypedPath;

use super::{entry, path};
use crate::file::{self, audio};
use crate::http::binary;
use crate::Error;

#[derive(Clone, o2o)]
#[ref_into(filesystem::Type)]
pub enum Impl<'fs> {
    #[type_hint(as Unit)]
    Local(Cow<'fs, super::local::Filesystem>),
    #[type_hint(as Unit)]
    S3(Cow<'fs, super::s3::Filesystem>),
}

impl Impl<'_> {
    pub fn path(&self) -> path::Builder {
        path::Builder(self.into())
    }

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
        sender: entry::Sender,
        prefix: Utf8TypedPath<'_>,
    ) -> Result<(), Error>;

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error>;
    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error>;

    async fn transcode_input(&self, path: Utf8TypedPath<'_>) -> Result<CString, Error>;
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
        sender: entry::Sender,
        prefix: Utf8TypedPath<'_>,
    ) -> Result<(), Error> {
        match self {
            Impl::Local(filesystem) => filesystem.scan_folder(sender, prefix).await,
            Impl::S3(filesystem) => filesystem.scan_folder(sender, prefix).await,
        }
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read(path).await,
            Impl::S3(filesystem) => filesystem.read(path).await,
        }
    }

    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.read_to_binary(source, offset).await,
            Impl::S3(filesystem) => filesystem.read_to_binary(source, offset).await,
        }
    }

    async fn transcode_input(&self, path: Utf8TypedPath<'_>) -> Result<CString, Error> {
        match self {
            Impl::Local(filesystem) => filesystem.transcode_input(path).await,
            Impl::S3(filesystem) => filesystem.transcode_input(path).await,
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

    use super::Trait as _;
    use crate::file::audio;
    use crate::filesystem::{entry, Entry};
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
    #[case(filesystem::Type::S3, "/nghe-backend-test-check-folder-bucket", true)]
    #[case(filesystem::Type::S3, "/nghe-backend-test-check-folder-bucket/test/", true)]
    #[tokio::test]
    async fn test_check_folder(
        #[future(awt)]
        #[with(0, 0, Some("nghe-backend-test-check-folder-bucket"))]
        mock: Mock,
        #[case] ty: filesystem::Type,
        #[case] path: &str,
        #[case] is_ok: bool,
    ) {
        let filesystem = mock.to_impl(ty);
        assert_eq!(filesystem.check_folder(path.into()).await.is_ok(), is_ok);
    }

    #[rstest]
    #[case(20, 10, 20, 15, 5)]
    #[case(50, 5, 15, 10, 10)]
    #[tokio::test]
    async fn test_scan_folder(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
        #[case] minimum_size: usize,
        #[case] n_txt: usize,
        #[case] n_dir: usize,
        #[case] n_smaller: usize,
        #[case] n_larger: usize,
    ) {
        let filesystem = mock.to_impl(ty);
        let prefix = filesystem.prefix().to_path_buf();
        let main_filesystem = filesystem.main().into_owned();

        let mut entries = vec![];

        for _ in 0..n_txt {
            let relative_path = filesystem.fake_path((0..3).fake()).with_extension("txt");
            let content = fake::vec![u8; 0..(2 * minimum_size)];
            filesystem.write(relative_path.to_path(), &content).await;
        }

        for _ in 0..n_dir {
            let relative_path = filesystem.fake_path((0..3).fake());
            filesystem.create_dir(relative_path.to_path()).await;
        }

        for _ in 0..n_dir {
            let relative_path = filesystem
                .fake_path((0..3).fake())
                .with_extension(Faker.fake::<audio::Format>().as_ref());
            filesystem.create_dir(relative_path.to_path()).await;
        }

        for _ in 0..n_smaller {
            let relative_path = filesystem
                .fake_path((0..3).fake())
                .with_extension(Faker.fake::<audio::Format>().as_ref());
            let content = fake::vec![u8; 0..minimum_size];
            filesystem.write(relative_path.to_path(), &content).await;
        }

        for _ in 0..n_larger {
            let format: audio::Format = Faker.fake();
            let relative_path = filesystem.fake_path((0..3).fake()).with_extension(format.as_ref());

            let content = fake::vec![u8; ((minimum_size + 1)..(2 * minimum_size)).fake::<usize>()];
            filesystem.write(relative_path.to_path(), &content).await;
            entries.push(Entry { format, path: prefix.join(relative_path), last_modified: None });
        }

        let (tx, rx) = crate::sync::channel(mock.config.filesystem.scan.channel_size);
        let sender = entry::Sender { tx, minimum_size };
        let scan_handle = tokio::spawn(async move {
            main_filesystem.scan_folder(sender, prefix.to_path()).await.unwrap();
        });
        let scanned_entries: Vec<_> = rx.into_stream().collect().await;
        scan_handle.await.unwrap();

        assert_eq!(scanned_entries.len(), n_larger);
        assert_eq!(
            scanned_entries.into_iter().sorted().collect_vec(),
            entries.into_iter().sorted().collect_vec()
        );
    }
}
