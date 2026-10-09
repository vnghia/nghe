use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Url(#[from] url::ParseError),
    #[error(transparent)]
    InvalidBearerToken(#[from] axum_extra::headers::authorization::InvalidBearerToken),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("{}", error.message)]
    Http { status_code: StatusCode, error: nghe_api::Error },
}
