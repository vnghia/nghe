use std::fs::Metadata;

use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use time::OffsetDateTime;
use typed_path::Utf8TypedPath;

use super::{entry, path};
use crate::file::{self, audio};
use crate::http::binary;
use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct Filesystem;

impl Filesystem {
    #[cfg(windows)]
    fn is_native(path: &Utf8TypedPath<'_>) -> bool {
        path.is_windows()
    }

    #[cfg(unix)]
    fn is_native(path: &Utf8TypedPath<'_>) -> bool {
        path.is_unix()
    }
}

impl super::Trait for Filesystem {
    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        if Self::is_native(&path)
            && path.is_absolute()
            && tokio::fs::metadata(path.as_str()).await?.is_dir()
        {
            Ok(())
        } else {
            Err(Error::InvalidParameter("Folder path must be absolute and be a directory"))
        }
    }

    async fn scan_folder(
        &self,
        sender: entry::Sender,
        prefix: Utf8TypedPath<'_>,
    ) -> Result<(), Error> {
        let mut stream = WalkDir::new(prefix.as_ref());

        while let Some(entry) = stream.next().await {
            match entry {
                Ok(entry) => match entry.metadata().await {
                    Ok(metadata) => {
                        if metadata.is_file() {
                            let path = entry
                                .path()
                                .into_os_string()
                                .into_string()
                                .map(path::Local::from_string)
                                .map_err(Error::FilesystemLocalNonUTF8PathEncountered)?;
                            sender.send(path, &metadata).await?;
                        }
                    }
                    Err(err) => tracing::error!(list_folder_local_metadata_err = ?err),
                },
                Err(err) => tracing::error!(list_folder_local_walk_err = ?err),
            }
        }

        Ok(())
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        tokio::fs::read(path.as_str()).await.map_err(Error::from)
    }

    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error> {
        binary::Response::from_local(source.path.to_path(), &source.property, offset, true, false)
            .await
    }
}

impl entry::Metadata for Metadata {
    fn size(&self) -> Result<usize, Error> {
        self.len().try_into().map_err(Error::from)
    }

    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error> {
        Ok(self.modified().ok().map(OffsetDateTime::from))
    }
}
