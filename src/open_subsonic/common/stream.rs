use std::convert::Infallible;
use std::io::SeekFrom;

use anyhow::{Ok, Result};
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::headers::{AcceptRanges, ContentLength, ContentRange};
use axum_extra::TypedHeader;
use flume::r#async::RecvStream;
use flume::Receiver;
use futures::StreamExt;
use mime_guess::Mime;
use tokio::io::{AsyncRead, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::utils::fs::LocalPath;
use crate::OSError;

struct RxStream(RecvStream<'static, Vec<u8>>);

impl futures::Stream for RxStream {
    type Item = Result<Vec<u8>, Infallible>;

    // TODO: remove this when axum::body does not require TryStream anymore.
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx).map(|t| t.map(Result::<_, Infallible>::Ok))
    }
}

pub struct StreamResponse {
    mime: Mime,
    offset: u64,
    size: u64,
    seekable: bool,
    body: Body,
}

impl StreamResponse {
    fn from_ext(ext: &str) -> Mime {
        mime_guess::from_ext(ext).first_or_octet_stream()
    }

    pub async fn try_from_path(
        path: impl AsRef<LocalPath>,
        offset: impl Into<Option<u64>>,
        size: impl Into<Option<u64>>,
        seekable: bool,
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
            seekable,
            file,
        ))
    }

    pub fn from_async_read(
        ext: &str,
        offset: u64,
        size: u64,
        seekable: bool,
        reader: impl AsyncRead + Send + Sync + 'static,
    ) -> Self {
        Self {
            mime: Self::from_ext(ext),
            offset,
            size,
            seekable,
            body: Body::from_stream(ReaderStream::new(reader)),
        }
    }

    pub fn from_rx(ext: &str, rx: Receiver<Vec<u8>>) -> Self {
        Self {
            mime: Self::from_ext(ext),
            offset: 0,
            size: 0,
            seekable: false,
            body: Body::from_stream(RxStream(rx.into_stream())),
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
        } else if self.seekable {
            (
                [(header::CONTENT_TYPE, self.mime.essence_str())],
                (TypedHeader(ContentLength(self.size))),
                (TypedHeader(AcceptRanges::bytes())),
                self.body,
            )
                .into_response()
        } else if self.size > 0 {
            (
                [(header::CONTENT_TYPE, self.mime.essence_str())],
                (TypedHeader(ContentLength(self.size))),
                self.body,
            )
                .into_response()
        } else {
            ([(header::CONTENT_TYPE, self.mime.essence_str())], self.body).into_response()
        }
    }
}
