use anyhow::Result;
use axum::extract::State;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(GetBasicUserIdsParams, admin);
add_axum_response!(GetBasicUserIdsBody);

pub async fn get_basic_user_ids(pool: &DatabasePool) -> Result<Vec<users::BasicUserId<'static>>> {
    users::table
        .select(users::BasicUserId::as_select())
        .get_results(&mut pool.get().await?)
        .await
        .map_err(anyhow::Error::from)
}

pub async fn get_basic_user_ids_handler(
    State(database): State<Database>,
    _: GetBasicUserIdsRequest,
) -> GetBasicUserIdsJsonResponse {
    Ok(axum::Json(
        GetBasicUserIdsBody {
            basic_user_ids: get_basic_user_ids(&database.pool)
                .await?
                .into_iter()
                .map(users::BasicUserId::into)
                .collect(),
        }
        .into(),
    ))
}
