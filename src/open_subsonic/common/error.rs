use super::response::ErrorConstantResponse;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;

use nghe_proc_macros::wrap_subsonic_response;

pub enum OpenSubsonicError {
    Generic { source: anyhow::Error },
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
}

#[derive(Debug, Default, Serialize)]
struct ActualError {
    code: u8,
    message: String,
}

#[wrap_subsonic_response(false)]
#[derive(Debug, Default, Serialize)]
struct ErrorResponse {
    error: ActualError,
}

fn error_to_json(code: u8, message: String) -> Json<ErrorResponse> {
    tracing::error!(message);
    Json(ErrorResponse {
        subsonic_response: ActualErrorResponse {
            error: ActualError { code, message },
            ..Default::default()
        },
    })
}

impl<E> From<E> for OpenSubsonicError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        OpenSubsonicError::Generic { source: err.into() }
    }
}

impl IntoResponse for OpenSubsonicError {
    fn into_response(self) -> Response {
        match self {
            OpenSubsonicError::Generic { source } => error_to_json(0, source.to_string()),
            OpenSubsonicError::BadRequest {} => {
                error_to_json(10, "required parameter is missing".to_owned())
            }
            OpenSubsonicError::Unauthorized {} => {
                error_to_json(40, "wrong username or password".to_owned())
            }
            OpenSubsonicError::Forbidden {} => error_to_json(
                50,
                "user is not authorized for the given operation".to_owned(),
            ),
            OpenSubsonicError::NotFound {} => {
                error_to_json(70, "the requested data was not found".to_owned())
            }
        }
        .into_response()
    }
}
