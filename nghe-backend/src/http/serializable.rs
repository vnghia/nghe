use axum::response::IntoResponse;
use nghe_api::common::SubsonicResponse;
use serde::Serialize;

use super::extract::auth::request::Type;

#[derive(Debug)]
pub struct Response<R> {
    ty: Type,
    body: R,
}

impl<R: Serialize> IntoResponse for Response<R> {
    fn into_response(self) -> axum::response::Response {
        match self.ty {
            Type::FORM => axum::Json(SubsonicResponse::new(self.body)).into_response(),
            Type::JSON => axum::Json(self.body).into_response(),
        }
    }
}
