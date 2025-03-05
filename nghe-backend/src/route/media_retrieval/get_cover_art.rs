use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::get_cover_art::Request;
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::file::{self, picture};
use crate::http::binary;
use crate::http::header::ToOffset;
use crate::{Error, config, error};

#[handler]
pub async fn handler(
    database: &Database,
    #[handler(header)] range: Option<Range>,
    config: config::CoverArt,
    request: Request,
) -> Result<binary::Response, Error> {
    let dir = &config.dir.ok_or_else(|| error::Kind::MissingCoverArtDirectoryConfig)?;
    let property = file::Property::query_cover_art(database, request.id).await?;
    let offset = range.map(|range| range.to_offset(property.size.into())).transpose()?;
    binary::Response::from_path_property(
        property.path(dir, picture::Picture::FILENAME),
        &property,
        offset,
        #[cfg(test)]
        None,
    )
    .await
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use axum::http::StatusCode;
    use axum_extra::headers::{CacheControl, ContentLength, HeaderMapExt};
    use binary::property::Trait as _;
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::file;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(#[future(awt)] mock: Mock) {
        let picture: picture::Picture = Faker.fake();
        let id = picture.upsert_mock(&mock, None::<&str>).await;

        let binary = handler(mock.database(), None, mock.config.cover_art.clone(), Request { id })
            .await
            .unwrap();

        let (status_code, headers, body) = binary.extract().await;
        let body_len: u64 = body.len().try_into().unwrap();

        assert_eq!(status_code, StatusCode::OK);

        assert_eq!(headers.typed_get::<ContentLength>().unwrap().0, body_len);
        assert_eq!(
            headers.typed_get::<CacheControl>().unwrap(),
            file::Property::<picture::Format>::cache_control()
        );

        let local_bytes: &[u8] = picture.data.as_ref();
        assert_eq!(body, local_bytes);
    }
}
