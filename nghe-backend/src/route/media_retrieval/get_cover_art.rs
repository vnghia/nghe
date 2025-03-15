use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::get_cover_art::Request;
use nghe_proc_macro::handler;

use crate::database::Database;
use crate::file::{self, picture};
use crate::http::binary;
use crate::http::header::ToOffset;
#[cfg(test)]
use crate::test::transcode::Status as TranscodeStatus;
use crate::{Error, config, error, resize};

#[handler]
pub async fn handler(
    database: &Database,
    #[handler(header)] range: Option<Range>,
    config: config::CoverArt,
    request: Request,
) -> Result<binary::Response, Error> {
    const FORMAT: picture::Format = picture::Format::WebP;

    let dir = &config.dir.ok_or_else(|| error::Kind::MissingCoverArtDirectoryConfig)?;
    let property = file::Property::query_cover_art(database, request.id).await?;
    let input = property.path(dir, picture::Picture::FILENAME);
    let offset = range.map(|range| range.to_offset(property.size.into())).transpose()?;

    if let Some(size) = request.size {
        let output = if let Some(cache_dir) = config.cache_dir {
            let output =
                property.replace(FORMAT).path_create_dir(cache_dir, size.to_string()).await?;
            let cache_exists = tokio::fs::try_exists(&output).await?;

            // Similar logics in the stream handler applies here.
            if cache_exists {
                return binary::Response::from_path(
                    output,
                    FORMAT,
                    offset,
                    #[cfg(test)]
                    TranscodeStatus::ServeCachedOutput,
                )
                .await;
            }
            Some(output)
        } else {
            None
        };

        #[cfg(test)]
        let transcode_status =
            if output.is_some() { TranscodeStatus::WithCache } else { TranscodeStatus::NoCache };

        let data = resize::Resizer::spawn(input, output, FORMAT, size).await?;
        binary::Response::from_memory(
            FORMAT,
            data,
            offset,
            #[cfg(test)]
            transcode_status,
        )
    } else {
        binary::Response::from_path_property(
            &input,
            &property,
            offset,
            #[cfg(test)]
            None,
        )
        .await
    }
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

        let binary = handler(
            mock.database(),
            None,
            mock.config.cover_art.clone(),
            Request { id, size: None },
        )
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
