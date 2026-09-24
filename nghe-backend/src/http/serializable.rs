use axum::response::IntoResponse;
use nghe_api::common::SubsonicResponse;
use serde::Serialize;

use super::extract::request;

#[derive(Debug)]
pub struct Response<R> {
    pub ty: request::Type,
    pub body: R,
}

impl<R: Serialize> IntoResponse for Response<R> {
    fn into_response(self) -> axum::response::Response {
        match self.ty {
            request::Type::Form => axum::Json(SubsonicResponse::new(self.body)).into_response(),
            request::Type::Json => axum::Json(self.body).into_response(),
        }
    }
}
