use std::fmt::Debug;
use std::io::Seek;

use tracing::instrument;
use typed_path::Utf8NativePath;

use crate::{error, Error};

pub struct Lock;

impl Lock {
    fn open_read(path: impl AsRef<Utf8NativePath>) -> Result<std::fs::File, Error> {
        if cfg!(windows) {
            // On Windows, the file must be open with write permissions to lock it.
            std::fs::OpenOptions::new().read(true).write(true).open(path.as_ref())
        } else {
            std::fs::OpenOptions::new().read(true).open(path.as_ref())
        }
        .map_err(Error::from)
    }

    #[instrument(err(Debug, level = "debug"))]
    pub fn lock_read(path: impl AsRef<Utf8NativePath> + Debug) -> Result<std::fs::File, Error> {
        let mut file = Self::open_read(path)?;
        // The read lock might be acquired with an empty file since creating and locking exclusively
        // a file are two separate operations. We need to check if the file is empty before trying
        // to acquiring the read lock. If the file is empty, don't lock it so the write lock
        // can be acquired by the process that has created this file.
        if file.seek(std::io::SeekFrom::End(0))? > 0 {
            if file.try_lock_shared()? {
                Ok(file)
            } else {
                error::Kind::FileAlreadyExclusivelyLocked.into()
            }
        } else {
            error::Kind::EmptyFileEncountered.into()
        }
    }

    #[instrument(err(Debug, level = "debug"))]
    pub fn lock_write(path: impl AsRef<Utf8NativePath> + Debug) -> Result<std::fs::File, Error> {
        let file = std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_ref())?;
        if file.try_lock()? { Ok(file) } else { error::Kind::FileAlreadyLocked.into() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Lock {
        pub fn lock_read_blocking(
            path: impl AsRef<Utf8NativePath>,
        ) -> Result<std::fs::File, Error> {
            let file = Self::open_read(path)?;
            file.lock_shared()?;
            Ok(file)
        }
    }
}
