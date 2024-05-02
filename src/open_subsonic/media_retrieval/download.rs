use anyhow::Result;
use axum::extract::State;
use axum::Extension;
use nghe_proc_macros::add_common_validate;
use uuid::Uuid;

use super::utils::get_song_download_info;
use crate::models::*;
use crate::open_subsonic::StreamResponse;
use crate::utils::fs::{FsTrait, LocalFs, S3Fs};
use crate::{Database, DatabasePool, ServerError};

add_common_validate!(DownloadParams, download);

pub async fn download(
    pool: &DatabasePool,
    local_fs: &LocalFs,
    s3_fs: Option<&S3Fs>,
    user_id: Uuid,
    song_id: Uuid,
) -> Result<StreamResponse> {
    let (music_folder_path, fs_type, song_relative_path, ..) =
        get_song_download_info(pool, user_id, song_id).await?;
    match fs_type {
        music_folders::FsType::Local => {
            local_fs.read_to_stream(LocalFs::join(music_folder_path, song_relative_path)).await
        }
        music_folders::FsType::S3 => {
            S3Fs::unwrap(s3_fs)?
                .read_to_stream(S3Fs::join(music_folder_path, song_relative_path))
                .await
        }
    }
}

pub async fn download_handler(
    State(database): State<Database>,
    Extension(local_fs): Extension<LocalFs>,
    Extension(s3_fs): Extension<Option<S3Fs>>,
    req: DownloadRequest,
) -> Result<StreamResponse, ServerError> {
    download(&database.pool, &local_fs, s3_fs.as_ref(), req.user_id, req.params.id)
        .await
        .map_err(ServerError)
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::*;
    use crate::utils::test::http::to_bytes;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_download() {
        let mut infra = Infra::new()
            .await
            .add_folder(music_folders::FsType::Local, true)
            .await
            .add_user(None)
            .await;
        infra.add_n_song(0, 1).await.scan(.., None).await;

        let download_bytes = to_bytes(
            download(
                infra.pool(),
                infra.fs.local(),
                infra.fs.s3_option(),
                infra.user_id(0),
                infra.song_ids(..).await[0],
            )
            .await
            .unwrap()
            .into_response(),
        )
        .await
        .to_vec();
        let local_bytes = infra.fs.read_song(&infra.song_fs_infos(..)[0]).await;
        assert_eq!(download_bytes, local_bytes);
    }
}
