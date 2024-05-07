use std::io::SeekFrom;
use std::pin::Pin;
use std::task::{Context, Poll};

use anyhow::{Ok, Result};
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::headers::{AcceptRanges, ContentLength, ContentRange};
use axum_extra::TypedHeader;
use futures::{Future, Stream};
use kanal::{AsyncReceiver, ReceiveError};
use mime_guess::Mime;
use tokio::io::{AsyncRead, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::utils::fs::LocalPath;
use crate::OSError;

struct RxStream(AsyncReceiver<Vec<u8>>);

impl Stream for RxStream {
    type Item = Result<Vec<u8>, ReceiveError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        std::pin::pin!(self.0.recv()).as_mut().poll(cx).map(Option::Some)
    }
}

pub struct StreamResponse {
    mime: Mime,
    offset: u64,
    size: u64,
    streamable: bool,
    body: Body,
}

impl StreamResponse {
    fn from_ext(ext: &str) -> Mime {
        mime_guess::from_ext(ext).first_or_octet_stream()
    }

    pub async fn try_from_path<P: AsRef<LocalPath>, O: Into<Option<u64>>, S: Into<Option<u64>>>(
        path: P,
        offset: O,
        size: S,
        streamable: bool,
    ) -> Result<Self> {
        let path = path.as_ref();
        let mut file = tokio::fs::File::open(path.as_str()).await?;
        let offset = if let Some(offset) = offset.into()
            && offset > 0
        {
            file.seek(SeekFrom::Start(offset)).await?
        } else {
            0
        };
        let size = if let Some(size) = size.into() { size } else { file.metadata().await?.len() };

        Ok(Self::from_async_read(
            path.extension().ok_or_else(|| {
                OSError::InvalidParameter("path does not have an extension".into())
            })?,
            offset,
            size,
            streamable,
            file,
        ))
    }

    pub fn from_async_read<R: AsyncRead + Send + Sync + 'static>(
        ext: &str,
        offset: u64,
        size: u64,
        streamable: bool,
        reader: R,
    ) -> Self {
        Self {
            mime: Self::from_ext(ext),
            offset,
            size,
            streamable,
            body: Body::from_stream(ReaderStream::new(reader)),
        }
    }

    pub fn from_rx(ext: &str, rx: AsyncReceiver<Vec<u8>>) -> Self {
        Self {
            mime: Self::from_ext(ext),
            offset: 0,
            size: 0,
            streamable: false,
            body: Body::from_stream(RxStream(rx)),
        }
    }
}

impl IntoResponse for StreamResponse {
    fn into_response(self) -> Response {
        if self.offset > 0 {
            (
                StatusCode::PARTIAL_CONTENT,
                [(header::CONTENT_TYPE, self.mime.essence_str())],
                (TypedHeader(ContentLength(self.size))),
                (TypedHeader(AcceptRanges::bytes())),
                (TypedHeader(
                    ContentRange::bytes(self.offset.., self.size)
                        .expect("failed to construct content-range request"),
                )),
                self.body,
            )
                .into_response()
        } else if self.streamable {
            (
                [(header::CONTENT_TYPE, self.mime.essence_str())],
                (TypedHeader(ContentLength(self.size))),
                (TypedHeader(AcceptRanges::bytes())),
                self.body,
            )
                .into_response()
        } else {
            (
                [(header::CONTENT_TYPE, self.mime.essence_str())],
                (TypedHeader(ContentLength(self.size))),
                self.body,
            )
                .into_response()
        }
    }
}
