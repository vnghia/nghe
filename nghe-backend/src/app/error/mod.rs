use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not parse request due to {0}")]
    BadRequest(&'static str),

    #[error("Could not login due to bad credentials")]
    Unauthenticated,
    #[error("You need to have {0} role to perform this action")]
    Unauthorized(&'static str),

    #[error("Internal server error")]
    Internal(#[from] color_eyre::Report),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unauthenticated => StatusCode::FORBIDDEN,
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, self.to_string()).into_response()
    }
}
