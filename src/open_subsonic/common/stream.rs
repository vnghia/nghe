use std::convert::Infallible;
use std::path::Path;

use anyhow::{Ok, Result};
use axum::body::Body;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use flume::r#async::RecvStream;
use flume::Receiver;
use futures::StreamExt;
use mime_guess::Mime;
use tokio_util::io::ReaderStream;

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

enum StreamType {
    File(ReaderStream<tokio::fs::File>),
    Rx(RxStream),
}

pub struct StreamResponse {
    mime: Mime,
    stream: StreamType,
}

impl StreamResponse {
    pub async fn try_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        let stream = ReaderStream::new(tokio::fs::File::open(&path).await?);
        Ok(Self { mime, stream: StreamType::File(stream) })
    }

    pub fn from_rx(ext: &str, rx: Receiver<Vec<u8>>) -> Self {
        let mime = mime_guess::from_ext(ext).first_or_octet_stream();
        Self { mime, stream: StreamType::Rx(RxStream(rx.into_stream())) }
    }
}

impl IntoResponse for StreamResponse {
    fn into_response(self) -> Response {
        let body = match self.stream {
            StreamType::File(f) => Body::from_stream(f),
            StreamType::Rx(rx) => Body::from_stream(rx),
        };
        ([(header::CONTENT_TYPE, self.mime.essence_str().to_owned())], body).into_response()
    }
}
