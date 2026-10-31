use std::pin::Pin;
use std::task::{Context, Poll};

use futures::StreamExt;
use time::OffsetDateTime;
use typed_path::{Utf8TypedPath, Utf8TypedPathBuf};

use super::{Trait, local, s3};
use crate::file::audio;
use crate::{Error, error};

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

pub enum Stream {
    Local(Pin<Box<<local::Filesystem as Trait>::EntryStream>>),
    S3(<s3::Filesystem as Trait>::EntryStream),
}

impl Entry {
    pub fn try_new(
        minimum_size: usize,
        path: Utf8TypedPathBuf,
        metadata: &impl Metadata,
    ) -> Result<Option<Self>, Error> {
        let size = metadata.size()?;
        if size <= minimum_size {
            Ok(None)
        } else {
            audio::Format::try_from(
                path.extension().ok_or_else(|| error::Kind::MissingPathExtension(path.clone()))?,
            )
            .ok()
            .map(|format| {
                Ok::<_, Error>(Entry { format, path, last_modified: metadata.last_modified()? })
            })
            .transpose()
        }
    }

    pub fn relative_path(&self, base: impl AsRef<str>) -> Result<Utf8TypedPath<'_>, Error> {
        self.path.strip_prefix(base).map_err(Error::from)
    }
}

impl Unpin for Stream {}

impl futures::Stream for Stream {
    type Item = Result<Entry, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.get_mut() {
            Self::Local(stream) => stream.poll_next_unpin(cx),
            Self::S3(stream) => stream.poll_next_unpin(cx),
        }
    }
}
