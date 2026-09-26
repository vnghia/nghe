use time::OffsetDateTime;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::Error;
use crate::file::audio;

pub trait Metadata {
    fn size(&self) -> Result<usize, Error>;
    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error>;
}

#[derive(Debug)]
#[cfg_attr(test, derive(educe::Educe))]
#[cfg_attr(test, educe(PartialEq, Eq, PartialOrd, Ord))]
pub struct Entry {
    pub format: audio::Format,
    pub path: Utf8TypedPathBuf,
    #[cfg_attr(test, educe(PartialEq(ignore)))]
    #[cfg_attr(test, educe(PartialOrd(ignore)))]
    pub last_modified: Option<OffsetDateTime>,
}

impl Entry {
    pub fn relative_path(&self, base: impl AsRef<str>) -> Result<Utf8TypedPath<'_>, Error> {
        self.path.strip_prefix(base).map_err(Error::from)
    }
}

pub struct Sender {
    pub tx: loole::Sender<Entry>,
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
            self.tx
                .send_async(Entry { format, path, last_modified: metadata.last_modified()? })
                .await?;
        }
        Ok(())
    }
}
