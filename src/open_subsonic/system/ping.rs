use super::super::common::response::{wrap_success_response_root, SuccessConstantResponse};
use axum::Json;
use serde::Serialize;

wrap_success_response_root!(PingResponse, {});

pub async fn ping() -> Json<PingResponse> {
    Json(PingResponse::default())
}
