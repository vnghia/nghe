use anyhow::Result;
use axum::extract::State;
use nghe_proc_macros::add_validate;
use uuid::Uuid;

use super::utils::get_song_download_info;
use crate::open_subsonic::StreamResponse;
use crate::{Database, DatabasePool, ServerError};

#[add_validate]
#[derive(Debug)]
pub struct DownloadParams {
    id: Uuid,
}

pub async fn download(pool: &DatabasePool, user_id: Uuid, song_id: Uuid) -> Result<StreamResponse> {
    let absolute_path = get_song_download_info(pool, user_id, song_id).await?;
    StreamResponse::from_path(&absolute_path).await
}

pub async fn download_handler(
    State(database): State<Database>,
    req: DownloadRequest,
) -> Result<StreamResponse, ServerError> {
    download(&database.pool, req.user_id, req.params.id).await.map_err(ServerError)
}
