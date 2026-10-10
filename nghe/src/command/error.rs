use std::fmt::Debug;

use color_eyre::Report;
use o2o::o2o;
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Kind {
    #[error("[{}][{}] {}", status_code, error.code as u8, error.message)]
    Http { status_code: StatusCode, error: nghe_api::Error },
}

#[derive(o2o)]
#[from_owned(url::ParseError| repeat(), return Report::from(@).into())]
#[from_owned(axum_extra::headers::authorization::InvalidBearerToken)]
#[from_owned(reqwest::Error)]
#[from_owned(serde_json::Error)]
#[from_owned(std::io::Error)]
pub struct Error {
    pub source: Report,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.source, f)
    }
}

impl From<Kind> for Error {
    fn from(source: Kind) -> Self {
        Self { source: source.into() }
    }
}

impl<T> From<Kind> for Result<T, Error> {
    fn from(value: Kind) -> Self {
        Err(value.into())
    }
}

impl From<Report> for Error {
    fn from(source: Report) -> Self {
        Self { source }
    }
}
