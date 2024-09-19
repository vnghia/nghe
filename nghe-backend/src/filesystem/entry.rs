use time::OffsetDateTime;
use typed_path::Utf8TypedPathBuf;

use super::Trait;
use crate::file::{audio, File};
use crate::Error;

pub trait Metadata {
    fn size(&self) -> Result<usize, Error>;
    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error>;
}

#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative))]
#[cfg_attr(test, derivative(PartialEq, Eq, PartialOrd, Ord))]
pub struct Entry {
    pub format: audio::Format,
    pub path: Utf8TypedPathBuf,
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    #[cfg_attr(test, derivative(PartialOrd = "ignore"))]
    #[cfg_attr(test, derivative(Ord = "ignore"))]
    pub last_modified: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct Filesystem<'a, 'fs> {
    filesystem: &'a super::Impl<'fs>,
    entry: Entry,
}

impl Entry {
    pub fn filesystem<'a, 'fs>(self, filesystem: &'a super::Impl<'fs>) -> Filesystem<'a, 'fs> {
        Filesystem { filesystem, entry: self }
    }
}

impl<'a, 'fs> Filesystem<'a, 'fs> {
    pub async fn read(&self) -> Result<File<audio::Format>, Error> {
        File::new(self.filesystem.read(self.entry.path.to_path()).await?, self.entry.format)
    }
}

pub struct Sender {
    pub tx: tokio::sync::mpsc::Sender<Entry>,
    pub minimum_size: usize,
}

impl Sender {
    pub async fn send(
        &self,
        path: Utf8TypedPathBuf,
        metadata: &impl Metadata,
    ) -> Result<(), Error> {
        let size = metadata.size()?;
        if size > self.minimum_size
            && let Some(extension) = path.extension()
            && let Ok(format) = audio::Format::try_from(extension)
        {
            self.tx.send(Entry { format, path, last_modified: metadata.last_modified()? }).await?;
        }
        Ok(())
    }
}
