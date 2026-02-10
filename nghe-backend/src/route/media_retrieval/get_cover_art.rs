use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::get_cover_art::Request;
use nghe_proc_macro::handler;
#[cfg(not(target_os = "linux"))]
use tokio::fs;
#[cfg(target_os = "linux")]
use uring_file::fs;

use crate::database::Database;
use crate::file::{self, image};
use crate::http::binary;
use crate::http::header::ToOffset;
#[cfg(test)]
use crate::test::binary::Status as BinaryStatus;
use crate::{Error, config, error};

#[handler]
pub async fn handler(
    database: &Database,
    config: config::CoverArt,
    #[handler(header)] range: Option<Range>,
    request: Request,
) -> Result<binary::Response, Error> {
    const FORMAT: image::Format = image::Format::WebP;

    let dir = &config.dir.ok_or_else(|| error::Kind::MissingCoverArtDirectoryConfig)?;
    let property = file::Property::query_cover_art(database, request.id).await?;
    let input = property.path(dir, image::Image::FILENAME);
    let offset = range.map(|range| range.to_offset(property.size.into())).transpose()?;

    if let Some(size) = request.size {
        let output = if let Some(cache_dir) = config.cache_dir {
            let output =
                property.replace(FORMAT).path_create_dir(cache_dir, size.to_string()).await?;
            let cache_exists = fs::exists(&output).await;

            // Similar logics in the stream handler applies here.
            if cache_exists {
                return binary::Response::from_path(
                    output,
                    FORMAT,
                    offset,
                    #[cfg(test)]
                    BinaryStatus::ServeCachedOutput,
                )
                .await;
            }
            Some(output)
        } else {
            None
        };

        #[cfg(test)]
        let binary_status =
            if output.is_some() { BinaryStatus::WithCache } else { BinaryStatus::NoCache };

        let data = image::Resizer::spawn(input, output, FORMAT, size).await?;
        binary::Response::from_memory(
            FORMAT,
            data,
            offset,
            #[cfg(test)]
            binary_status,
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
    use std::io::Cursor;

    use ::image::ImageReader;
    use axum::http::StatusCode;
    use axum_extra::headers::{CacheControl, ContentLength, HeaderMapExt};
    use binary::property::Trait as _;
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;
    use crate::file;
    use crate::test::binary::Header as BinaryHeader;
    use crate::test::{Mock, mock};

    async fn spawn_resize(
        mock: &Mock,
        n_task: usize,
        request: Request,
    ) -> (Vec<(StatusCode, Vec<u8>)>, Vec<BinaryStatus>) {
        let mut stream_set = tokio::task::JoinSet::new();
        for _ in 0..n_task {
            let database = mock.database().clone();
            let config = mock.config.cover_art.clone();
            stream_set.spawn(async move {
                handler(&database, config, None, request).await.unwrap().extract().await
            });
        }
        let (responses, binary_status): (Vec<_>, Vec<_>) = stream_set
            .join_all()
            .await
            .into_iter()
            .map(|(status, headers, body)| {
                ((status, body), headers.typed_get::<BinaryHeader>().unwrap().0)
            })
            .unzip();
        (responses, binary_status.into_iter().sorted().collect())
    }

    #[rstest]
    #[tokio::test]
    async fn test_handler(#[future(awt)] mock: Mock) {
        let image: image::Image = Faker.fake();
        let id = image.upsert_mock(&mock, None::<&str>).await;

        let binary = handler(
            mock.database(),
            mock.config.cover_art.clone(),
            None,
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
            file::Property::<image::Format>::cache_control()
        );

        let local_bytes: &[u8] = image.data.as_ref();
        assert_eq!(body, local_bytes);
    }

    #[rstest]
    #[tokio::test]
    async fn test_resize(#[future(awt)] mock: Mock) {
        let image: image::Image = Faker.fake();
        let id = image.upsert_mock(&mock, None::<&str>).await;

        let size = 50;
        let request = Request { id, size: Some(size) };

        let (responses, binary_status) = spawn_resize(&mock, 1, request).await;
        for (status, body) in responses {
            assert_eq!(status, StatusCode::OK);
            assert!(
                ImageReader::new(Cursor::new(body)).with_guessed_format().unwrap().decode().is_ok()
            );
        }
        assert_eq!(binary_status, &[BinaryStatus::WithCache]);

        let (responses, binary_status) = spawn_resize(&mock, 2, request).await;
        for (status, body) in responses {
            assert_eq!(status, StatusCode::OK);
            assert!(
                ImageReader::new(Cursor::new(body)).with_guessed_format().unwrap().decode().is_ok()
            );
        }
        assert_eq!(
            binary_status,
            &[BinaryStatus::ServeCachedOutput, BinaryStatus::ServeCachedOutput]
        );
    }
}
