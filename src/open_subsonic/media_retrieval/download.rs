use super::utils::get_song_download_info;
use crate::{
    open_subsonic::common::binary_response::BinaryResponse, Database, DatabasePool, ServerError,
};

use anyhow::Result;
use axum::extract::State;
use nghe_proc_macros::add_validate;
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct DownloadParams {
    id: Uuid,
}

pub async fn download(pool: &DatabasePool, user_id: Uuid, song_id: Uuid) -> Result<BinaryResponse> {
    let (absolute_path, format) = get_song_download_info(pool, user_id, song_id).await?;
    let data = tokio::fs::read(absolute_path).await?;
    Ok(BinaryResponse { format, data })
}

pub async fn download_handler(
    State(database): State<Database>,
    req: DownloadRequest,
) -> Result<BinaryResponse, ServerError> {
    download(&database.pool, req.user.id, req.params.id)
        .await
        .map_err(ServerError)
}
