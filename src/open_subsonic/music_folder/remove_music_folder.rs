use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(RemoveMusicFolderParams, admin);
add_axum_response!(RemoveMusicFolderBody);

pub async fn remove_music_folder(pool: &DatabasePool, id: Uuid) -> Result<()> {
    diesel::delete(music_folders::table.filter(music_folders::id.eq(id)))
        .execute(&mut pool.get().await?)
        .await?;

    Ok(())
}

pub async fn remove_music_folder_handler(
    State(database): State<Database>,
    req: RemoveMusicFolderRequest,
) -> RemoveMusicFolderJsonResponse {
    remove_music_folder(&database.pool, req.params.id).await?;
    Ok(axum::Json(RemoveMusicFolderBody {}.into()))
}
