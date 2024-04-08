use anyhow::Result;
use axum::extract::State;
use nghe_proc_macros::{add_common_convert, add_common_validate};
use uuid::Uuid;

use super::utils::get_song_download_info;
use crate::open_subsonic::StreamResponse;
use crate::{Database, DatabasePool, ServerError};

#[add_common_convert]
#[derive(Debug)]
pub struct DownloadParams {
    id: Uuid,
}
add_common_validate!(DownloadParams, download);

pub async fn download(pool: &DatabasePool, user_id: Uuid, song_id: Uuid) -> Result<StreamResponse> {
    let (absolute_path, ..) = get_song_download_info(pool, user_id, song_id).await?;
    StreamResponse::try_from_path(&absolute_path).await
}

pub async fn download_handler(
    State(database): State<Database>,
    req: DownloadRequest,
) -> Result<StreamResponse, ServerError> {
    download(&database.pool, req.user_id, req.params.id).await.map_err(ServerError)
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::*;
    use crate::utils::test::http::to_bytes;
    use crate::utils::test::Infra;

    #[tokio::test]
    async fn test_download() {
        let mut infra = Infra::new().await.n_folder(1).await.add_user(None).await;
        infra.add_n_song(0, 1).scan(.., None).await;

        let download_bytes = to_bytes(
            download(infra.pool(), infra.user_id(0), infra.song_ids(..).await[0])
                .await
                .unwrap()
                .into_response(),
        )
        .await
        .to_vec();
        let local_bytes = std::fs::read(infra.song_fs_infos(..)[0].absolute_path()).unwrap();
        assert_eq!(download_bytes, local_bytes);
    }
}
