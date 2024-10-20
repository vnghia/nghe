use std::time::Duration;

use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::body::AsyncReadBody;
use axum_extra::headers::{
    AcceptRanges, CacheControl, ContentLength, ContentRange, ETag, HeaderMapExt,
};
use tokio::io::{AsyncRead, AsyncSeekExt, SeekFrom};
use typed_path::Utf8TypedPath;

use crate::{file, Error};

pub struct Binary {
    status: StatusCode,
    header: HeaderMap,
    body: AsyncReadBody,
}

impl Binary {
    const MAX_AGE: Duration = Duration::from_secs(31_536_000);

    pub fn new(
        status: StatusCode,
        header: HeaderMap,
        reader: impl AsyncRead + Send + 'static,
    ) -> Self {
        Self { status, header, body: AsyncReadBody::new(reader) }
    }

    pub async fn from_local<F: file::Mime>(
        path: Utf8TypedPath<'_>,
        property: file::Property<F>,
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

    pub fn from_async_read<F: file::Mime>(
        reader: impl AsyncRead + Send + 'static,
        property: file::Property<F>,
        offset: impl Into<Option<u64>>,
        seekable: bool,
        cacheable: bool,
    ) -> Result<Self, Error> {
        let mut header = HeaderMap::new();

        header.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(property.mime()));

        let size = property.size.into();
        header.typed_insert(ContentLength(size));
        header.typed_insert(
            property.hash.to_string().parse::<ETag>().map_err(color_eyre::Report::from)?,
        );

        let offset = offset.into().unwrap_or(0);
        header.typed_insert(ContentRange::bytes(offset.., size).map_err(color_eyre::Report::from)?);

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

impl IntoResponse for Binary {
    fn into_response(self) -> Response {
        (self.status, self.header, self.body).into_response()
    }
}
