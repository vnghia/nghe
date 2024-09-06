use aws_sdk_s3::primitives::ByteStreamError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use lofty::error::LoftyError;

use crate::media::file;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    InvalidParameter(&'static str),
    #[error("Could not serialize request due to {0}")]
    SerializeRequest(&'static str),
    #[error(transparent)]
    ExtractRequestBody(#[from] axum::extract::rejection::BytesRejection),

    #[error("Could not checkout a connection from connection pool")]
    CheckoutConnectionPool,
    #[error("Could not decrypt value from database")]
    DecryptDatabaseValue,

    #[error("{0}")]
    Unauthorized(&'static str),
    #[error("Could not login due to bad credentials")]
    Unauthenticated,
    #[error("You need to have {0} role to perform this action")]
    MissingRole(&'static str),

    #[error("Could not parse date from {0:?}")]
    MediaDateFormat(String),
    #[error(
        "Could not parse position from track number {track_number:?}, track total \
         {track_total:?}, disc number {disc_number:?} and disc total {disc_total:?}"
    )]
    MediaPositionFormat {
        track_number: Option<String>,
        track_total: Option<String>,
        disc_number: Option<String>,
        disc_total: Option<String>,
    },
    #[error("There should not be more musicbrainz id than artist name")]
    MediaArtistMbzIdMoreThanArtistName,
    #[error("Song artist should not be empty")]
    MediaSongArtistEmpty,
    #[error("Could not read vorbis comments from flac file")]
    MediaFlacMissingVorbisComments,
    #[error("File with {0:?} are not supported")]
    MediaFileTypeNotSupported(file::Type),

    #[error(transparent)]
    Internal(#[from] color_eyre::Report),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, status_message) = match &self {
            Error::InvalidParameter(_) | Error::SerializeRequest(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            Error::ExtractRequestBody(_) => {
                (StatusCode::BAD_REQUEST, "Could not extract request body".into())
            }
            Error::Unauthenticated => (StatusCode::FORBIDDEN, self.to_string()),
            Error::Unauthorized(_) | Error::MissingRole(_) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
        };
        (status_code, status_message).into_response()
    }
}

impl From<diesel::result::Error> for Error {
    fn from(value: diesel::result::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<isolang::ParseLanguageError> for Error {
    fn from(value: isolang::ParseLanguageError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<uuid::Error> for Error {
    fn from(value: uuid::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl From<LoftyError> for Error {
    fn from(value: LoftyError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<ByteStreamError> for Error {
    fn from(value: ByteStreamError) -> Self {
        Self::Internal(value.into())
    }
}
