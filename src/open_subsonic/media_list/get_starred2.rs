use crate::Database;

use axum::extract::State;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::Serialize;

#[add_validate]
#[derive(Debug)]
pub struct GetStarred2Params {}

#[derive(Serialize)]
pub struct Starred2Result {}

#[wrap_subsonic_response]
pub struct Starred2Body {
    starred2: Starred2Result,
}

pub async fn get_starred2_handler(
    State(_): State<Database>,
    _: GetStarred2Request,
) -> Starred2JsonResponse {
    Starred2Body {
        starred2: Starred2Result {},
    }
    .into()
}
