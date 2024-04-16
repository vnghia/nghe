use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetAllowedUsersParams, admin);
add_axum_response!(GetAllowedUsersBody);

pub async fn get_allowed_users(pool: &DatabasePool, id: Uuid) -> Result<Vec<Uuid>> {
    users::table
        .inner_join(user_music_folder_permissions::table)
        .filter(user_music_folder_permissions::music_folder_id.eq(id))
        .select(users::id)
        .get_results::<Uuid>(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_allowed_users_handler(
    State(database): State<Database>,
    req: GetAllowedUsersRequest,
) -> GetAllowedUsersJsonResponse {
    Ok(axum::Json(
        GetAllowedUsersBody { ids: get_allowed_users(&database.pool, req.params.id).await? }.into(),
    ))
}
