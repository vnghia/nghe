use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::download::Request;
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::file::{self, audio};
use crate::filesystem::{self, Filesystem, Trait};
use crate::http::binary;
use crate::http::header::ToOffset;

pub async fn handler_impl(
    filesystem: filesystem::Impl<'_>,
    source: binary::Source<file::Property<audio::Format>>,
    offset: Option<u64>,
) -> Result<binary::Response, Error> {
    filesystem.read_to_binary(&source, offset).await
}

#[handler]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    #[handler(header)] range: Option<Range>,
    user_id: Uuid,
    request: Request,
) -> Result<binary::Response, Error> {
    let (filesystem, source) =
        binary::Source::audio(database, filesystem, user_id, request.id).await?;
    let offset = range.map(|range| range.to_offset(source.property.size.into())).transpose()?;
    handler_impl(filesystem, source, offset).await
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use axum::http::StatusCode;
    use axum_extra::headers::{
        AcceptRanges, CacheControl, ContentLength, ContentRange, ETag, HeaderMapExt,
    };
    use binary::property::Trait as _;
    use nghe_api::common::filesystem;
    use rstest::rstest;
    use xxhash_rust::xxh3::xxh3_64;

    use super::*;
    use crate::file::audio;
    use crate::http::header::ToETag;
    use crate::test::{Mock, mock};

    #[rstest]
    #[case(None)]
    #[case(Some(0))]
    #[case(Some(500))]
    #[tokio::test]
    async fn test_download(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
        #[values(filesystem::Type::Local, filesystem::Type::S3)] ty: filesystem::Type,
        #[values(true, false)] allow: bool,
        #[case] offset: Option<u64>,
    ) {
        mock.add_music_folder().ty(ty).allow(allow).call().await;
        let mut music_folder = mock.music_folder(0).await;
        music_folder.add_audio_filesystem::<&str>().format(audio::Format::Flac).call().await;

        let local_bytes =
            music_folder.to_impl().read(music_folder.absolute_path(0).to_path()).await.unwrap();
        let local_hash = xxh3_64(&local_bytes);
        let local_bytes = &local_bytes[offset.unwrap_or(0).try_into().unwrap()..];

        let range = offset.map(|offset| Range::bytes(offset..).unwrap());
        let user_id = mock.user_id(0).await;
        let request = Request { id: music_folder.song_id_filesystem(0).await };
        let binary = handler(mock.database(), mock.filesystem(), range, user_id, request).await;

        assert_eq!(binary.is_ok(), allow);

        if allow {
            let binary = binary.unwrap();
            let (status, headers, body) = binary.extract().await;

            let body_len: u64 = body.len().try_into().unwrap();
            let offset = offset.unwrap_or(0);

            assert_eq!(
                status,
                if offset == 0 { StatusCode::OK } else { StatusCode::PARTIAL_CONTENT }
            );

            assert_eq!(headers.typed_get::<ContentLength>().unwrap().0, body_len);
            assert_eq!(
                headers.typed_get::<ContentRange>().unwrap(),
                ContentRange::bytes(offset.., Some(offset + body_len)).unwrap()
            );
            assert_eq!(headers.typed_get::<ETag>().unwrap(), local_hash.to_etag().unwrap());
            assert_eq!(headers.typed_get::<AcceptRanges>().unwrap(), AcceptRanges::bytes());
            assert_eq!(
                headers.typed_get::<CacheControl>().unwrap(),
                file::Property::<audio::Format>::cache_control()
            );

            assert_eq!(body, local_bytes);
        }
    }
}
