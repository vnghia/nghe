use std::fs::Metadata;

use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use time::OffsetDateTime;
use typed_path::Utf8TypedPath;

use super::{entry, path};
use crate::file::{self, audio};
use crate::http::binary;
use crate::{error, Error};

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
        if !Self::is_native(&path) {
            error::Kind::InvalidTypedPathPlatform(path.to_path_buf()).into()
        } else if !path.is_absolute() {
            error::Kind::InvalidAbsolutePath(path.to_path_buf()).into()
        } else if !tokio::fs::metadata(path.as_str()).await?.is_dir() {
            error::Kind::InvalidDirectoryPath(path.to_path_buf()).into()
        } else {
            Ok(())
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
                                .map_err(error::Kind::NonUTF8PathEncountered)?;
                            sender.send(path, &metadata).await?;
                        }
                    }
                    Err(error) => tracing::error!(list_folder_local_metadata_error = ?error),
                },
                Err(error) => tracing::error!(list_folder_local_walk_err = ?error),
            }
        }

        Ok(())
    }

    async fn exists(&self, path: Utf8TypedPath<'_>) -> Result<bool, Error> {
        tokio::fs::try_exists(path.as_str()).await.map_err(Error::from)
    }

    async fn read(&self, path: Utf8TypedPath<'_>) -> Result<Vec<u8>, Error> {
        tokio::fs::read(path.as_str()).await.map_err(Error::from)
    }

    async fn read_to_binary(
        &self,
        source: &binary::Source<file::Property<audio::Format>>,
        offset: Option<u64>,
    ) -> Result<binary::Response, Error> {
        let path = match source.path.to_path() {
            Utf8TypedPath::Unix(path) => path.with_platform_encoding(),
            Utf8TypedPath::Windows(path) => path.with_platform_encoding(),
        };

        binary::Response::from_path_property(
            path,
            &source.property,
            offset,
            #[cfg(test)]
            None,
        )
        .await
    }

    async fn transcode_input(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        Ok(path.as_str().to_owned())
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
