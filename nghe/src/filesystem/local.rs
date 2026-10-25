use std::fs::Metadata;

use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use time::OffsetDateTime;
use tokio::io::{AsyncSeekExt, SeekFrom};
use typed_path::{Utf8PlatformPath, Utf8PlatformPathBuf, Utf8TypedPath};

use super::{entry, path};
use crate::{Error, error};

#[derive(Debug, Clone, Copy)]
pub struct Filesystem;

impl Filesystem {
    #[cfg(windows)]
    fn is_native(path: Utf8TypedPath<'_>) -> bool {
        path.is_windows()
    }

    #[cfg(unix)]
    fn is_native(path: Utf8TypedPath<'_>) -> bool {
        path.is_unix()
    }

    pub fn to_platform(path: Utf8TypedPath<'_>) -> Result<Utf8PlatformPathBuf, Error> {
        match path {
            Utf8TypedPath::Unix(unix_path) => {
                if cfg!(unix) {
                    Ok(unix_path.with_platform_encoding())
                } else {
                    error::Kind::InvalidTypedPathPlatform(path.to_path_buf()).into()
                }
            }
            Utf8TypedPath::Windows(windows_path) => {
                if cfg!(windows) {
                    Ok(windows_path.with_platform_encoding())
                } else {
                    error::Kind::InvalidTypedPathPlatform(path.to_path_buf()).into()
                }
            }
        }
    }

    pub async fn open_with_offset(
        path: impl AsRef<Utf8PlatformPath>,
        offset: impl Into<Option<u64>> + Copy,
    ) -> Result<tokio::fs::File, Error> {
        let mut file = tokio::fs::File::open(path.as_ref()).await?;
        if let Some(offset) = offset.into()
            && offset > 0
        {
            file.seek(SeekFrom::Start(offset)).await?;
        }
        Ok(file)
    }
}

impl super::Trait for Filesystem {
    type AsyncRead = tokio::io::BufReader<tokio::fs::File>;

    async fn check_folder(&self, path: Utf8TypedPath<'_>) -> Result<(), Error> {
        if !Self::is_native(path) {
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

    async fn read_to_string(&self, path: Utf8TypedPath<'_>) -> Result<String, Error> {
        tokio::fs::read_to_string(path.as_str()).await.map_err(Error::from)
    }

    async fn read_to_async_read(
        &self,
        path: Utf8TypedPath<'_>,
        offset: Option<u64>,
    ) -> Result<Self::AsyncRead, Error> {
        let path = Self::to_platform(path)?;
        Ok(tokio::io::BufReader::new(Self::open_with_offset(path, offset).await?))
    }

    fn transcode_input(
        &self,
        path: Utf8TypedPath<'_>,
    ) -> impl Future<Output = Result<String, Error>> {
        std::future::ready(Ok(path.as_str().to_owned()))
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
