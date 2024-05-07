use std::ops::Bound;

use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use axum_extra::headers::Range;
use axum_extra::TypedHeader;
use nghe_proc_macros::add_common_validate;
use uuid::Uuid;

use super::utils::get_song_download_info;
use crate::models::*;
use crate::open_subsonic::StreamResponse;
use crate::utils::fs::{FsTrait, LocalFs, S3Fs};
use crate::{Database, DatabasePool, OSError, ServerError};

add_common_validate!(DownloadParams, download);

pub async fn download(
    pool: &DatabasePool,
    local_fs: &LocalFs,
    s3_fs: Option<&S3Fs>,
    range: Option<Range>,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<StreamResponse> {
    let (music_folder_path, fs_type, song_relative_path, _, song_file_size) =
        get_song_download_info(pool, user_id, song_id).await?;
    let size = song_file_size as _;
    let offset = if let Some(range) = range {
        if let Bound::Included(offset) = range
            .satisfiable_ranges(size)
            .next()
            .ok_or_else(|| OSError::InvalidParameter("range header".into()))?
            .0
        {
            offset
        } else {
            anyhow::bail!(OSError::InvalidParameter("range header start".into()))
        }
    } else {
        0
    };
    match fs_type {
        music_folders::FsType::Local => {
            local_fs
                .read_to_stream(LocalFs::join(music_folder_path, song_relative_path), offset, size)
                .await
        }
        music_folders::FsType::S3 => {
            S3Fs::unwrap(s3_fs)?
                .read_to_stream(S3Fs::join(music_folder_path, song_relative_path), offset, size)
                .await
        }
    }
}

pub async fn download_handler(
    State(database): State<Database>,
    Extension(local_fs): Extension<LocalFs>,
    Extension(s3_fs): Extension<Option<S3Fs>>,
    range: Option<TypedHeader<Range>>,
    req: DownloadRequest,
) -> Result<StreamResponse, ServerError> {
    let range = range.map(|h| h.0);
    download(&database.pool, &local_fs, s3_fs.as_ref(), range, req.user_id, req.params.id)
        .await
        .map_err(ServerError)
}

#[cfg(test)]
mod tests {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;
    use concat_string::concat_string;
    use strum::IntoEnumIterator;

    use super::*;
    use crate::utils::test::http::to_bytes;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_download() {
        for fs_type in music_folders::FsType::iter() {
            let mut infra = Infra::new().await.add_folder(fs_type, true).await.add_user(None).await;
            infra.add_n_song(0, 1).await.scan(.., None).await;
            let raw_bytes = infra.fs.read_song(&infra.song_fs_infos(..)[0]).await;

            let response = download(
                infra.pool(),
                infra.fs.local(),
                infra.fs.s3_option(),
                None,
                infra.user_id(0),
                infra.song_ids(..).await[0],
            )
            .await
            .unwrap()
            .into_response();

            let headers = response.headers().clone();
            let size = headers
                .get(header::CONTENT_LENGTH)
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let accept = headers.get(header::ACCEPT_RANGES).unwrap().to_str().unwrap();
            let download_bytes = to_bytes(response).await.to_vec();

            assert_eq!(size, raw_bytes.len(), "{:?} failed for downloading", fs_type);
            assert_eq!(accept, "bytes", "{:?} failed for downloading", fs_type);
            assert_eq!(download_bytes, raw_bytes, "{:?} failed for downloading", fs_type);
        }
    }

    #[tokio::test]
    async fn test_download_range() {
        for fs_type in music_folders::FsType::iter() {
            let offset = 100;
            let mut infra = Infra::new().await.add_folder(fs_type, true).await.add_user(None).await;
            infra.add_n_song(0, 1).await.scan(.., None).await;
            let raw_bytes = infra.fs.read_song(&infra.song_fs_infos(..)[0]).await;

            let response = download(
                infra.pool(),
                infra.fs.local(),
                infra.fs.s3_option(),
                Some(Range::bytes(offset..).unwrap()),
                infra.user_id(0),
                infra.song_ids(..).await[0],
            )
            .await
            .unwrap()
            .into_response();

            let status = response.status();
            let headers = response.headers().clone();
            let size = headers
                .get(header::CONTENT_LENGTH)
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            let accept = headers.get(header::ACCEPT_RANGES).unwrap().to_str().unwrap();
            let range = headers.get(header::CONTENT_RANGE).unwrap().to_str().unwrap();
            let download_bytes = to_bytes(response).await.to_vec();

            assert_eq!(
                status,
                StatusCode::PARTIAL_CONTENT,
                "{:?} failed for downloading range",
                fs_type
            );
            assert_eq!(size, raw_bytes.len(), "{:?} failed for downloading range", fs_type);
            assert_eq!(accept, "bytes", "{:?} failed for downloading range", fs_type);
            assert_eq!(
                range,
                concat_string!(
                    "bytes ",
                    offset.to_string(),
                    "-",
                    (raw_bytes.len() - 1).to_string(),
                    "/",
                    raw_bytes.len().to_string()
                ),
                "{:?} failed for downloading range",
                fs_type
            );
            assert_eq!(
                &download_bytes,
                &raw_bytes[offset as _..],
                "{:?} failed for downloading range",
                fs_type
            );
        }
    }
}
