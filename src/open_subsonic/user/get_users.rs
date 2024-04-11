use anyhow::Result;
use axum::extract::State;
use diesel::query_dsl::methods::SelectDsl;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetUsersParams, admin);
add_axum_response!(GetUsersBody);

async fn get_users(pool: &DatabasePool) -> Result<Vec<users::User>> {
    users::table
        .select(users::User::as_select())
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_users_handler(
    State(database): State<Database>,
    _: GetUsersRequest,
) -> GetUsersJsonResponse {
    Ok(axum::Json(
        GetUsersBody {
            users: get_users(&database.pool).await?.into_iter().map(users::User::into).collect(),
        }
        .into(),
    ))
}
