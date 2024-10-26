use axum_extra::headers::Range;
pub use nghe_api::media_retrieval::download::Request;
use nghe_proc_macro::handler;
use uuid::Uuid;

use super::offset;
use crate::database::Database;
use crate::filesystem::{Filesystem, Trait};
use crate::response::{binary, Binary};
use crate::Error;

#[handler(role = download, headers = [range])]
pub async fn handler(
    database: &Database,
    filesystem: &Filesystem,
    range: Option<Range>,
    user_id: Uuid,
    request: Request,
) -> Result<Binary, Error> {
    let (filesystem, source) =
        binary::Source::audio(database, filesystem, user_id, request.id).await?;
    let offset = offset::from_range(range, source.property.size.into())?;
    filesystem.read_to_binary(&source, offset).await
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_extra::headers::{AcceptRanges, ContentRange, HeaderMapExt};
    use nghe_api::common::filesystem;
    use rstest::rstest;

    use super::*;
    use crate::file::audio;
    use crate::test::{mock, Mock};

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
            &music_folder.to_impl().read(music_folder.absolute_path(0).to_path()).await.unwrap()
                [offset.unwrap_or(0).try_into().unwrap()..];

        let range = offset.map(|offset| Range::bytes(offset..).unwrap());
        let user_id = mock.user(0).await.user.id;
        let request = Request { id: music_folder.query_id(0).await };
        let binary = handler(mock.database(), mock.filesystem(), range, user_id, request).await;

        assert_eq!(binary.is_ok(), allow);

        if allow {
            let binary = binary.unwrap();
            let (status, headers, body) = binary.extract().await;

            let offset = offset.unwrap_or(0);
            assert_eq!(
                status,
                if offset == 0 { StatusCode::OK } else { StatusCode::PARTIAL_CONTENT }
            );
            assert_eq!(headers.typed_get::<AcceptRanges>().unwrap(), AcceptRanges::bytes());
            assert_eq!(
                headers.typed_get::<ContentRange>().unwrap(),
                ContentRange::bytes(offset.., Some(offset + u64::try_from(body.len()).unwrap()))
                    .unwrap()
            );
            assert_eq!(body, local_bytes);
        }
    }
}