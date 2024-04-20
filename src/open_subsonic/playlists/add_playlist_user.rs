use anyhow::Result;
use axum::extract::State;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use nghe_types::playlists::add_playlist_user::AddPlaylistUserParams;
use uuid::Uuid;

use super::utils::check_access_level;
use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(AddPlaylistUserParams);
add_axum_response!(AddPlaylistUserBody);

pub async fn add_playlist_user(
    pool: &DatabasePool,
    admin_id: Uuid,
    params: AddPlaylistUserParams,
) -> Result<()> {
    check_access_level(pool, admin_id, playlists_users::AccessLevel::Admin).await?;

    diesel::insert_into(playlists_users::table)
        .values::<playlists_users::AddUser>(params.into())
        .execute(&mut pool.get().await?)
        .await?;
    Ok(())
}

pub async fn add_playlist_user_handler(
    State(database): State<Database>,
    req: AddPlaylistUserRequest,
) -> AddPlaylistUserJsonResponse {
    add_playlist_user(&database.pool, req.user_id, req.params).await?;
    Ok(axum::Json(AddPlaylistUserBody {}.into()))
}
