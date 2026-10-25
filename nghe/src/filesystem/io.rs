use std::io::Result as IoResult;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::{Trait, local, s3};

pub enum AsyncRead {
    Local(<local::Filesystem as Trait>::AsyncRead),
    S3(<s3::Filesystem as Trait>::AsyncRead),
}

impl From<<local::Filesystem as Trait>::AsyncRead> for AsyncRead {
    fn from(value: <local::Filesystem as Trait>::AsyncRead) -> Self {
        Self::Local(value)
    }
}

impl Unpin for AsyncRead {}

impl tokio::io::AsyncRead for AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        match self.get_mut() {
            Self::Local(reader) => Pin::new(reader).poll_read(cx, buf),
            Self::S3(reader) => Pin::new(reader).poll_read(cx, buf),
        }
    }
}

pub enum Read {
    Local(<local::Filesystem as Trait>::Read),
    S3(<s3::Filesystem as Trait>::Read),
}

impl From<<local::Filesystem as Trait>::Read> for Read {
    fn from(value: <local::Filesystem as Trait>::Read) -> Self {
        Self::Local(value)
    }
}

impl std::io::Read for Read {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self {
            Read::Local(reader) => reader.read(buf),
            Read::S3(reader) => reader.read(buf),
        }
    }
}
