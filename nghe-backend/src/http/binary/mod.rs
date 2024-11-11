pub mod property;
pub mod source;

use std::convert::Infallible;

use axum::body::Body;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_extra::headers::{
    AcceptRanges, ContentLength, ContentRange, HeaderMapExt, TransferEncoding,
};
use futures_lite::{Stream, StreamExt};
use loole::{Receiver, RecvStream};
pub use source::Source;
use tokio::io::{AsyncRead, AsyncSeekExt, SeekFrom};
use tokio_util::io::ReaderStream;
use typed_path::Utf8TypedPath;

use crate::Error;

struct RxStream(RecvStream<Vec<u8>>);

#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    header: HeaderMap,
    body: Body,
}

impl RxStream {
    fn new(rx: Receiver<Vec<u8>>) -> Self {
        Self(rx.into_stream())
    }
}

impl Stream for RxStream {
    type Item = Result<Vec<u8>, Infallible>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next(cx).map(|t| t.map(Result::Ok))
    }
}

impl From<RxStream> for Body {
    fn from(value: RxStream) -> Self {
        Self::from_stream(value)
    }
}

impl Response {
    fn new<P: property::Trait>(
        body: Body,
        property: &P,
        offset: impl Into<Option<u64>>,
    ) -> Result<Self, Error> {
        let mut header = HeaderMap::new();

        header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(property.mime()));

        let status = if let Some(size) = property.size() {
            let offset = offset.into().unwrap_or(0);
            header.typed_insert(ContentLength(size - offset));
            header.typed_insert(
                ContentRange::bytes(offset.., size).map_err(color_eyre::Report::from)?,
            );
            if offset == 0 { StatusCode::OK } else { StatusCode::PARTIAL_CONTENT }
        } else {
            header.typed_insert(TransferEncoding::chunked());
            StatusCode::OK
        };

        if let Some(etag) = property.etag()? {
            header.typed_insert(etag);
        }

        if P::SEEKABLE {
            header.typed_insert(AcceptRanges::bytes());
        }

        header.typed_insert(P::cache_control());

        Ok(Self { status, header, body })
    }

    pub async fn from_local(
        path: Utf8TypedPath<'_>,
        property: &impl property::Trait,
        offset: impl Into<Option<u64>> + Copy,
    ) -> Result<Self, Error> {
        let mut file = tokio::fs::File::open(path.as_str()).await?;
        if let Some(offset) = offset.into()
            && offset > 0
        {
            file.seek(SeekFrom::Start(offset)).await?;
        }
        Self::from_async_read(file, property, offset)
    }

    pub fn from_async_read(
        reader: impl AsyncRead + Send + 'static,
        property: &impl property::Trait,
        offset: impl Into<Option<u64>>,
    ) -> Result<Self, Error> {
        Self::new(Body::from_stream(ReaderStream::new(reader)), property, offset)
    }

    pub fn from_rx(
        rx: Receiver<Vec<u8>>,
        property: &impl property::Trait,
        offset: impl Into<Option<u64>>,
    ) -> Result<Self, Error> {
        Self::new(RxStream::new(rx).into(), property, offset)
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.header, self.body).into_response()
    }
}

#[cfg(test)]
mod test {
    use http_body_util::BodyExt;

    use super::*;

    impl Response {
        pub async fn extract(self) -> (StatusCode, HeaderMap, Vec<u8>) {
            let response = self.into_response();
            let status = response.status();
            let header = response.headers().clone();
            let body = response.into_body().collect().await.unwrap().to_bytes().to_vec();
            (status, header, body)
        }
    }
}
