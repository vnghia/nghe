use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not parse request due to {0}")]
    BadRequest(&'static str),
    #[error(transparent)]
    ExtractRequestBody(#[from] axum::extract::rejection::BytesRejection),

    #[error("Could not checkout a connection from connection pool")]
    CheckoutConnectionPool,
    #[error("Could not decrypt value from database")]
    DecryptDatabaseValue,

    #[error("Could not login due to bad credentials")]
    Unauthenticated,
    #[error("You need to have {0} role to perform this action")]
    Unauthorized(&'static str),

    #[error(transparent)]
    Internal(#[from] color_eyre::Report),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, status_message) = match &self {
            Error::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::ExtractRequestBody(_) => {
                (StatusCode::BAD_REQUEST, "Could not extract request body".into())
            }
            Error::Unauthenticated => (StatusCode::FORBIDDEN, self.to_string()),
            Error::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Internal(_) | Error::CheckoutConnectionPool | Error::DecryptDatabaseValue => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };
        (status_code, status_message).into_response()
    }
}
