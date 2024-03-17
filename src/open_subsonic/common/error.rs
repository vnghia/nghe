use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

use nghe_proc_macros::wrap_subsonic_response;
use std::borrow::Cow;

#[derive(Debug, Error)]
pub enum OSError {
    // Generic
    #[error("{0} not found")]
    NotFound(Cow<'static, str>),
    #[error("{0} parameter is invalid")]
    InvalidParameter(Cow<'static, str>),

    // Request
    #[error(transparent)]
    BadRequest(#[from] axum_extra::extract::FormRejection),

    // User
    #[error("Wrong username or password")]
    Unauthorized,
    #[error("User is forbidden to {0}")]
    Forbidden(Cow<'static, str>),
}

#[derive(Debug)]
pub struct ServerError(pub anyhow::Error);
pub type ServerResponse<T> = Result<T, ServerError>;
pub type ServerJsonResponse<T> = ServerResponse<Json<T>>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActualError {
    code: u8,
    message: String,
}

#[wrap_subsonic_response(success = false)]
#[derive(Debug)]
struct ErrorBody {
    error: ActualError,
}

fn to_error_response(code: u8, err: &anyhow::Error) -> ErrorJsonResponse {
    let message = err.to_string();
    tracing::error!(err = &message);
    ErrorBody {
        error: ActualError { code, message },
    }
    .into()
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let code = match self.0.root_cause().downcast_ref::<OSError>() {
            Some(err) => match err {
                OSError::NotFound(_) => 70,
                OSError::BadRequest(_) | OSError::InvalidParameter(_) => 10,
                OSError::Unauthorized => 40,
                OSError::Forbidden(_) => 50,
            },
            None => 0,
        };
        to_error_response(code, &self.0).into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
