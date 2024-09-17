use time::OffsetDateTime;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use crate::media::audio;
use crate::Error;

pub trait Metadata {
    fn size(&self) -> Result<usize, Error>;
    fn last_modified(&self) -> Result<Option<OffsetDateTime>, Error>;
}

#[derive(Debug)]
#[cfg_attr(test, derive(derivative::Derivative))]
#[cfg_attr(test, derivative(PartialEq, Eq, PartialOrd, Ord))]
pub struct Entry {
    pub ty: audio::Type,
    pub relative_path: Utf8TypedPathBuf,
    pub size: usize,
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
    #[cfg_attr(test, derivative(PartialOrd = "ignore"))]
    #[cfg_attr(test, derivative(Ord = "ignore"))]
    pub last_modified: Option<OffsetDateTime>,
}

pub struct Sender {
    pub tx: tokio::sync::mpsc::Sender<Entry>,
    pub minimum_size: usize,
}

impl Sender {
    pub async fn send(
        &self,
        prefix: impl AsRef<str>,
        path: Utf8TypedPath<'_>,
        metadata: &impl Metadata,
    ) -> Result<(), Error> {
        let size = metadata.size()?;
        if size > self.minimum_size
            && let Some(extension) = path.extension()
            && let Ok(ty) = audio::Type::try_from(extension)
        {
            let relative_path = path.strip_prefix(prefix)?.to_path_buf();
            self.tx
                .send(Entry { ty, relative_path, size, last_modified: metadata.last_modified()? })
                .await?;
        }
        Ok(())
    }
}
