use super::response::ErrorConstantResponse;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;

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

#[derive(Debug, Default, Serialize)]
struct ActualErrorResponse {
    error: ActualError,

    #[serde(flatten)]
    constant: ErrorConstantResponse,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    #[serde(rename = "subsonic-response")]
    subsonic_response: ActualErrorResponse,
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
