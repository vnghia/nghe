use super::super::SuccessConstantResponse;
use axum::Json;
use serde::Serialize;

use nghe_proc_macros::wrap_subsonic_response;

#[wrap_subsonic_response]
#[derive(Debug, Default, Serialize)]
pub struct PingResponse {}

pub async fn ping() -> Json<PingResponse> {
    Json(PingResponse::default())
}
