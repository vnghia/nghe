use std::borrow::Cow;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use nghe_proc_macros::add_axum_response;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OSError {
    // Generic
    #[error("{0} not found")]
    NotFound(Cow<'static, str>),
    #[error("{0} parameter is invalid")]
    InvalidParameter(Cow<'static, str>),

    // IOError
    #[error(transparent)]
    IOError(#[from] std::io::Error),

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

add_axum_response!(ErrorBody);

fn to_error_response(code: u8, err: &anyhow::Error) -> ErrorJsonResponse {
    let message = err.to_string();
    tracing::error!("{:?}", err);
    Ok(axum::Json(ErrorBody { error: ActualError { code, message } }.into()))
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status_code, error_code) = match self.0.root_cause().downcast_ref::<OSError>() {
            Some(err) => match err {
                OSError::NotFound(_) | OSError::IOError(_) => (StatusCode::NOT_FOUND, 70),
                OSError::BadRequest(_) | OSError::InvalidParameter(_) => {
                    (StatusCode::BAD_REQUEST, 10)
                }
                OSError::Unauthorized => (StatusCode::UNAUTHORIZED, 40),
                OSError::Forbidden(_) => (StatusCode::FORBIDDEN, 50),
            },
            None => (StatusCode::INTERNAL_SERVER_ERROR, 0),
        };
        (status_code, to_error_response(error_code, &self.0)).into_response()
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
