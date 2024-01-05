use super::response::ErrorConstantResponse;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum OpenSubsonicError {
    #[snafu(whatever, display("{message}"))]
    Generic {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[snafu(display("Required parameter is missing"))]
    BadRequest {},

    #[snafu(display("Wrong username or password"))]
    Unauthorized {},

    #[snafu(display("User is not authorized for the given operation"))]
    Forbidden {},

    #[snafu(display("The requested data was not found"))]
    NotFound {},
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

fn error_to_json(code: u8, error: &OpenSubsonicError) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        subsonic_response: ActualErrorResponse {
            error: ActualError {
                code: code,
                message: error.to_string(),
            },
            ..Default::default()
        },
    })
}

impl IntoResponse for OpenSubsonicError {
    fn into_response(self) -> Response {
        match self {
            OpenSubsonicError::Generic {
                message: _,
                ref source,
            } => {
                tracing::error!(source);
                error_to_json(0, &self)
            }
            OpenSubsonicError::BadRequest {} => error_to_json(10, &self),
            OpenSubsonicError::Unauthorized {} => error_to_json(40, &self),
            OpenSubsonicError::Forbidden {} => error_to_json(50, &self),
            OpenSubsonicError::NotFound {} => error_to_json(70, &self),
        }
        .into_response()
    }
}
