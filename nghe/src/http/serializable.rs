use std::fmt::Debug;

use axum::response::IntoResponse;
use nghe_api::http::{ErrorSubsonicResponse, SubsonicResponse};
use serde::Serialize;

use super::extract::request;
use crate::{Error, error};

#[derive(Debug)]
pub struct Response<R> {
    pub ty: request::Type,
    pub body: R,
}

pub struct ErrorResponse {
    pub ty: request::Type,
    pub error: Error,
}

impl Debug for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl<R: Serialize> IntoResponse for Response<R> {
    fn into_response(self) -> axum::response::Response {
        match self.ty {
            request::Type::Form => axum::Json(SubsonicResponse::new(self.body)).into_response(),
            request::Type::Json => axum::Json(self.body).into_response(),
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let error = nghe_api::Error {
            code: self.error.open_subsonic_code,
            message: self.error.source.to_string(),
        };

        match self.ty {
            request::Type::Form => {
                (self.error.status_code, axum::Json(ErrorSubsonicResponse::new(error)))
                    .into_response()
            }
            request::Type::Json => (self.error.status_code, axum::Json(error)).into_response(),
        }
    }
}

impl<E: Into<error::Kind>> From<(request::Type, E)> for ErrorResponse {
    fn from((ty, error): (request::Type, E)) -> Self {
        Self { ty, error: error.into().into() }
    }
}

impl From<(request::Type, Error)> for ErrorResponse {
    fn from((ty, error): (request::Type, Error)) -> Self {
        Self { ty, error }
    }
}

impl<E: Into<error::Kind>> From<E> for ErrorResponse {
    fn from(value: E) -> Self {
        (request::Type::Json, value.into()).into()
    }
}

impl<T> From<ErrorResponse> for Result<T, ErrorResponse> {
    fn from(value: ErrorResponse) -> Self {
        Err(value)
    }
}
