use anyhow::Result;
use axum::extract::State;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_axum_response, add_common_validate};
use uuid::Uuid;

use crate::models::*;
use crate::{Database, DatabasePool};

add_common_validate!(DeleteUserParams, admin);
add_axum_response!(DeleteUserBody);

async fn delete_user(pool: &DatabasePool, id: Uuid) -> Result<()> {
    diesel::delete(users::table.filter(users::id.eq(id))).execute(&mut pool.get().await?).await?;
    Ok(())
}

pub async fn delete_user_handler(
    State(database): State<Database>,
    req: DeleteUserRequest,
) -> DeleteUserJsonResponse {
    delete_user(&database.pool, req.params.id).await?;
    Ok(axum::Json(DeleteUserBody {}.into()))
}
