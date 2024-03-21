use anyhow::{Ok, Result};
use axum::{
    body::Body,
    http::header,
    response::{IntoResponse, Response},
};
use crossfire::{
    channel::{self, MPSCShared},
    mpsc,
};
use futures::StreamExt;
use mime_guess::Mime;
use std::{convert::Infallible, path::Path};
use tokio_util::io::ReaderStream;

struct RxStream<S: MPSCShared>(channel::Stream<Vec<u8>, mpsc::RxFuture<Vec<u8>, S>>);

impl<S: MPSCShared> futures::Stream for RxStream<S> {
    type Item = Result<Vec<u8>, Infallible>;

    // TODO: remove this when axum::body does not require TryStream anymore.
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0
            .poll_next_unpin(cx)
            .map(|t| t.map(Result::<_, Infallible>::Ok))
    }
}

enum StreamType {
    File(ReaderStream<tokio::fs::File>),
    Rx(RxStream<mpsc::SharedSenderBRecvF>),
}

pub struct StreamResponse {
    mime: Mime,
    stream: StreamType,
}

impl StreamResponse {
    pub async fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        let stream = ReaderStream::new(tokio::fs::File::open(path).await?);
        Ok(Self {
            mime,
            stream: StreamType::File(stream),
        })
    }

    pub fn from_rx(ext: &str, rx: mpsc::RxFuture<Vec<u8>, mpsc::SharedSenderBRecvF>) -> Self {
        let mime = mime_guess::from_ext(ext).first_or_octet_stream();
        Self {
            mime,
            stream: StreamType::Rx(RxStream(rx.into_stream())),
        }
    }
}

impl IntoResponse for StreamResponse {
    fn into_response(self) -> Response {
        let body = match self.stream {
            StreamType::File(f) => Body::from_stream(f),
            StreamType::Rx(rx) => Body::from_stream(rx),
        };
        (
            [(header::CONTENT_TYPE, self.mime.essence_str().to_owned())],
            body,
        )
            .into_response()
    }
}
