use super::utils::get_song_absolute_path;
use crate::{Database, DatabasePool, ServerError};

use anyhow::Result;
use axum::extract::State;
use nghe_proc_macros::add_validate;
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct DownloadParams {
    id: Uuid,
}

pub async fn download(pool: &DatabasePool, user_id: Uuid, song_id: Uuid) -> Result<Vec<u8>> {
    let song_absolute_path = get_song_absolute_path(pool, user_id, song_id).await?;
    tokio::fs::read(song_absolute_path)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn download_handler(
    State(database): State<Database>,
    req: DownloadRequest,
) -> Result<Vec<u8>, ServerError> {
    download(&database.pool, req.user.id, req.params.id)
        .await
        .map_err(ServerError)
}
