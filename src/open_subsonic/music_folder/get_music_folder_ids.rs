use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetMusicFolderIdsParams, admin);
add_axum_response!(GetMusicFolderIdsBody);

async fn get_music_folder_ids(pool: &DatabasePool) -> Result<Vec<Uuid>> {
    music_folders::table
        .select(music_folders::id)
        .order(music_folders::name.asc())
        .get_results::<Uuid>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_music_folder_ids_handler(
    State(database): State<Database>,
    _: GetMusicFolderIdsRequest,
) -> GetMusicFolderIdsJsonResponse {
    Ok(axum::Json(
        GetMusicFolderIdsBody { ids: get_music_folder_ids(&database.pool).await? }.into(),
    ))
}
