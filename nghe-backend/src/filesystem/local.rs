use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use time::OffsetDateTime;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use super::Entry;
use crate::media::file;
use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct Filesystem;

impl Filesystem {
    #[cfg(windows)]
    fn is_native(path: &Utf8TypedPath<'_>) -> bool {
        path.is_windows()
    }

    #[cfg(windows)]
    fn from_native(path: impl AsRef<str>) -> Utf8TypedPathBuf {
        Utf8TypedPathBuf::from_windows(path)
    }

    #[cfg(unix)]
    fn is_native(path: &Utf8TypedPath<'_>) -> bool {
        path.is_unix()
    }

    #[cfg(unix)]
    fn from_native(path: impl AsRef<str>) -> Utf8TypedPathBuf {
        Utf8TypedPathBuf::from_unix(path)
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
        path: Utf8TypedPath<'_>,
        minimum_size: usize,
        tx: tokio::sync::mpsc::Sender<super::Entry>,
    ) -> Result<(), Error> {
        let mut stream = WalkDir::new(path.as_ref());

        while let Some(entry) = stream.next().await {
            match entry {
                Ok(entry) => match entry.metadata().await {
                    Ok(metadata) => {
                        let size = metadata.len().try_into()?;
                        if metadata.is_file() && size >= minimum_size {
                            let path = entry
                                .path()
                                .into_os_string()
                                .into_string()
                                .map(Self::from_native)
                                .map_err(Error::NonUTF8PathEncountered)?;
                            if let Some(extension) = path.extension()
                                && let Ok(file_type) = file::Type::try_from(extension)
                            {
                                tx.send(Entry {
                                    file_type,
                                    path,
                                    size,
                                    last_modified: metadata
                                        .modified()
                                        .ok()
                                        .map(OffsetDateTime::from),
                                })
                                .await?;
                            }
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
}
