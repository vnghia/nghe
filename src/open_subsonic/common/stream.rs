use std::convert::Infallible;

use anyhow::{Ok, Result};
use axum::body::Body;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use flume::r#async::RecvStream;
use flume::Receiver;
use futures::StreamExt;
use mime_guess::Mime;
use tokio::io::AsyncRead;
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
    body: Body,
}

impl StreamResponse {
    fn from_ext(ext: &str) -> Mime {
        mime_guess::from_ext(ext).first_or_octet_stream()
    }

    pub async fn try_from_path<P: AsRef<LocalPath>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        Ok(Self::from_async_read(
            path.extension().ok_or_else(|| {
                OSError::InvalidParameter("path does not have an extension".into())
            })?,
            tokio::fs::File::open(path.as_str()).await?,
        ))
    }

    pub fn from_async_read<R: AsyncRead + Send + Sync + 'static>(ext: &str, reader: R) -> Self {
        Self { mime: Self::from_ext(ext), body: Body::from_stream(ReaderStream::new(reader)) }
    }

    pub fn from_rx(ext: &str, rx: Receiver<Vec<u8>>) -> Self {
        Self { mime: Self::from_ext(ext), body: Body::from_stream(RxStream(rx.into_stream())) }
    }
}

impl IntoResponse for StreamResponse {
    fn into_response(self) -> Response {
        ([(header::CONTENT_TYPE, self.mime.essence_str().to_owned())], self.body).into_response()
    }
}
