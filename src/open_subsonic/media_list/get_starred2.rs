use axum::extract::State;
use nghe_proc_macros::{add_axum_response, add_common_validate};

use crate::Database;

add_common_validate!(GetStarred2Params);
add_axum_response!(Starred2Body);

pub async fn get_starred2_handler(
    State(_): State<Database>,
    _: GetStarred2Request,
) -> Starred2JsonResponse {
    Ok(axum::Json(Starred2Body { starred2: Starred2Result {} }.into()))
}
