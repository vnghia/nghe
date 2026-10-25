use std::io::Result as IoResult;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::{Trait, local, s3};

pub enum AsyncRead {
    Local(<local::Filesystem as Trait>::AsyncRead),
    S3(<s3::Filesystem as Trait>::AsyncRead),
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

impl tokio::io::AsyncBufRead for AsyncRead {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<&[u8]>> {
        match self.get_mut() {
            Self::Local(reader) => Pin::new(reader).poll_fill_buf(cx),
            Self::S3(reader) => Pin::new(reader).poll_fill_buf(cx),
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        match self.get_mut() {
            Self::Local(reader) => Pin::new(reader).consume(amt),
            Self::S3(reader) => Pin::new(reader).consume(amt),
        }
    }
}

pub enum Read {
    Local(<local::Filesystem as Trait>::Read),
    S3(<s3::Filesystem as Trait>::Read),
}

impl std::io::Read for Read {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self {
            Read::Local(reader) => reader.read(buf),
            Read::S3(reader) => reader.read(buf),
        }
    }
}

impl std::io::BufRead for Read {
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        match self {
            Read::Local(reader) => reader.fill_buf(),
            Read::S3(reader) => reader.fill_buf(),
        }
    }

    fn consume(&mut self, amount: usize) {
        match self {
            Read::Local(reader) => reader.consume(amount),
            Read::S3(reader) => reader.consume(amount),
        }
    }
}
