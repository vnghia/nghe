pub mod source;

use std::time::Duration;

use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum_extra::body::AsyncReadBody;
use axum_extra::headers::{AcceptRanges, CacheControl, ContentLength, ContentRange, HeaderMapExt};
use nghe_api::common::format;
pub use source::{Property, Source};
use tokio::io::{AsyncRead, AsyncSeekExt, SeekFrom};
use typed_path::Utf8TypedPath;

use crate::Error;

#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    header: HeaderMap,
    body: AsyncReadBody,
}

impl Response {
    const MAX_AGE: Duration = Duration::from_secs(31_536_000);

    pub fn new(
        status: StatusCode,
        header: HeaderMap,
        reader: impl AsyncRead + Send + 'static,
    ) -> Self {
        Self { status, header, body: AsyncReadBody::new(reader) }
    }

    pub async fn from_local<F: format::Format>(
        path: Utf8TypedPath<'_>,
        property: &Property<F>,
        offset: impl Into<Option<u64>> + Copy,
        seekable: bool,
        cacheable: bool,
    ) -> Result<Self, Error> {
        let mut file = tokio::fs::File::open(path.as_str()).await?;
        if let Some(offset) = offset.into()
            && offset > 0
        {
            file.seek(SeekFrom::Start(offset)).await?;
        }
        Self::from_async_read(file, property, offset, seekable, cacheable)
    }

    pub fn from_async_read<F: format::Format>(
        reader: impl AsyncRead + Send + 'static,
        property: &Property<F>,
        offset: impl Into<Option<u64>>,
        seekable: bool,
        cacheable: bool,
    ) -> Result<Self, Error> {
        let mut header = HeaderMap::new();

        header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(property.mime()));

        let offset = offset.into().unwrap_or(0);
        let size: u64 = property.size.into();
        header.typed_insert(ContentLength(size - offset));
        header.typed_insert(ContentRange::bytes(offset.., size).map_err(color_eyre::Report::from)?);

        if let Some(etag) = property.etag()? {
            header.typed_insert(etag);
        }

        if seekable {
            header.typed_insert(AcceptRanges::bytes());
        }

        header.typed_insert(if cacheable {
            CacheControl::new().with_private().with_immutable().with_max_age(Self::MAX_AGE)
        } else {
            CacheControl::new().with_no_cache()
        });

        Ok(Self::new(
            if offset == 0 { StatusCode::OK } else { StatusCode::PARTIAL_CONTENT },
            header,
            reader,
        ))
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
