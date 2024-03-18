use super::{
    utils::get_song_download_info, ServeMusicFolderResponse, ServeMusicFolderResult,
    ServeMusicFolders,
};
use crate::{Database, DatabasePool, ServerError};

use axum::{extract::State, Extension};
use nghe_proc_macros::add_validate;
use uuid::Uuid;

#[add_validate]
#[derive(Debug)]
pub struct DownloadParams {
    id: Uuid,
}

pub async fn download(
    pool: &DatabasePool,
    serve_music_folders: &mut ServeMusicFolders,
    user_id: Uuid,
    song_id: Uuid,
) -> ServeMusicFolderResult {
    let (mf_id, relative_path) = get_song_download_info(pool, user_id, song_id).await?;
    serve_music_folders
        .get_mut(&mf_id)
        .expect("it it impossible to have no music folder with the given id")
        .call(&relative_path)
        .await
}

pub async fn download_handler(
    State(database): State<Database>,
    Extension(mut serve_music_folders): Extension<ServeMusicFolders>,
    req: DownloadRequest,
) -> ServeMusicFolderResponse {
    download(
        &database.pool,
        &mut serve_music_folders,
        req.user_id,
        req.params.id,
    )
    .await
    .map_err(ServerError)
}
