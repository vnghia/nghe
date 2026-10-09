#![allow(unused_variables)]

use std::fmt::Debug;

use axum::http::StatusCode;
use color_eyre::Report;
use nghe_api::error::OpenSubsonicCode;
use o2o::o2o;

use crate::file::audio;

#[derive(Debug, thiserror::Error, o2o)]
#[ref_into(StatusCode)]
#[ref_into(OpenSubsonicCode)]
pub enum Kind {
    // Request error
    #[error(transparent)]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    DeserializeBytes(#[from] axum::extract::rejection::BytesRejection),
    #[error(transparent)]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    ExtractForm(#[from] axum::extract::rejection::RawFormRejection),
    #[error(transparent)]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    DeserializeForm(#[from] serde_html_form::de::Error),
    #[error(transparent)]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    DeserializeJson(#[from] serde_json::Error),

    #[error("Missing request body")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    MissingRequestBody,
    #[error("Method not allowed: {0}")]
    #[into(StatusCode| StatusCode::METHOD_NOT_ALLOWED)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MethodNotAllowed(axum::http::Method),
    #[error("Missing content type header")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    MissingContentTypeHeader,
    #[error("Unsupported content type {0}")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    UnsupportedContentType(axum_extra::headers::ContentType),
    #[error("Missing authentication header")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    MissingAuthenticationHeader,
    #[error("Invalid bearer authorization format")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    InvalidBearerAuthorizationFormat,
    #[error("Wrong username or password")]
    #[into(StatusCode| StatusCode::UNAUTHORIZED)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::WrongUsernameOrPassword)]
    WrongUsernameOrPassword,
    #[error("Invalid API key")]
    #[into(StatusCode| StatusCode::UNAUTHORIZED)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::InvalidApiKey)]
    InvalidApiKey,
    #[error("User is not authorized for the given operation")]
    #[into(StatusCode| StatusCode::FORBIDDEN)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::UserIsNotAuthorizedForTheGivenOperation)]
    Forbidden,

    #[error("Invalid range header {0:?}")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    InvalidRangeHeader(axum_extra::headers::Range),

    #[error("Found more time than id in scrobble artist")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    InvalidScrobbleTimeSize,

    // Database error
    #[error("Could not decrypt database value")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    DatabaseValueDecryptionFailed,
    #[error("Invalid database config format for key {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidDatabaseConfigFomat(&'static str),
    #[error("Database corruption detected")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    DatabaseCorruptionDetected,

    // Filesystem error
    #[error("Missing extension in path {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingPathExtension(typed_path::Utf8TypedPathBuf),
    #[error("Missing parent in path {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingPathParent(typed_path::Utf8TypedPathBuf),
    #[error("Path {0} does not have correct encoding")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidTypedPathPlatform(typed_path::Utf8TypedPathBuf),
    #[error("Path {0} is not an absolute path")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidAbsolutePath(typed_path::Utf8TypedPathBuf),
    #[error("Path {0} is not a directory path")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidDirectoryPath(typed_path::Utf8TypedPathBuf),
    #[error("Non UTF-8 path encountered: {0:?}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    NonUTF8PathEncountered(std::ffi::OsString),

    #[error("Missing size in file/object metadata")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingFileSize,
    #[error("Empty file encountered")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    EmptyFileEncountered,

    // Media error
    #[error("Could not found vorbis comments in format {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingVorbisComments(audio::Format),
    #[error("Could not found id3v2 tag in format {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingId3V2Tag(audio::Format),

    #[error("Missing media name")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingMediaName,
    #[error("Missing song artist name")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingSongArtistName,
    #[error("Invalid artist name format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidArtistNameFormat,
    #[error("Found more musicbrainz id than artist name")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidMbzIdSize,

    #[error("Invalid date tag format with value {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidDateTagFormat(String),
    #[error("Invalid musicbrainz id tag format with value {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidMbzIdTagFormat(String),
    #[error(transparent)]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidLanguageTagFormat(#[from] isolang::ParseLanguageError),
    #[error(
        "Could not parse position from track number {track_number:?}, track total \
         {track_total:?}, disc number {disc_number:?} and disc total {disc_total:?}"
    )]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidPositionTagFormat {
        track_number: Option<String>,
        track_total: Option<String>,
        disc_number: Option<String>,
        disc_total: Option<String>,
    },

    #[error("Invalid id3v2 frame id config format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidId3v2FrameIdConfigFormat,
    #[error("Invalid id3v2 frame id config type")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidId3v2FrameIdConfigType,

    // Image error
    #[error("Missing image format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingImageFormat,
    #[error("Unsupported image format {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    UnsupportedImageFormat(String),

    #[error("Missing cover art directory config")]
    #[into(StatusCode| StatusCode::NOT_FOUND)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::TheRequestedDataWasNotFound)]
    MissingCoverArtDirectoryConfig,

    // Lyrics error
    #[error("Could not parse lyrics from {0}")]
    #[into(StatusCode| StatusCode::NOT_FOUND)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::TheRequestedDataWasNotFound)]
    InvalidLyricsLrcFormat(String),

    // Rspotify error
    #[error("Invalid spotify id format with value {0}")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    InvalidSpotifyIdFormat(String),

    // LastFM Error
    #[error("Could not build LastFM request URL")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::RequiredParameterIsMissing)]
    BuildLastFMRequestURLFailed,

    // Transcode error
    #[error("No audio track found in media")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingAudioTrack,
    #[error("Missing encoder codec")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingEncoderCodec,
    #[error("Missing av filter with name {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    MissingAVFilter(&'static str),

    // Various error
    #[error("Invalid index ignore prefixes format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::AGenericError)]
    InvalidIndexIgnorePrefixesFormat,

    #[error("The requested data was not found")]
    #[into(StatusCode| StatusCode::NOT_FOUND)]
    #[into(OpenSubsonicCode| OpenSubsonicCode::TheRequestedDataWasNotFound)]
    NotFound,
}

#[derive(o2o)]
#[from_owned(
    diesel_async::pooled_connection::deadpool::PoolError| repeat(),
    return Report::from(@).into()
)]
#[from_owned(std::string::FromUtf8Error)]
#[from_owned(std::num::TryFromIntError)]
#[from_owned(time::error::ComponentRange)]
#[from_owned(time::error::ConversionRange)]
#[from_owned(time::error::Parse)]
#[from_owned(lofty::error::FileParseError)]
#[from_owned(lofty::error::TagParseError)]
// TODO: remove this after https://github.com/Serial-ATA/lofty-rs/issues/728
#[from_owned(lofty::id3::v2::error::FrameParseError)]
#[from_owned(reqwest::header::ToStrError)]
#[from_owned(typed_path::StripPrefixError)]
#[from_owned(tokio::task::JoinError)]
#[from_owned(ffmpeg_next::Error)]
#[from_owned(tokio::sync::AcquireError)]
#[from_owned(std::ffi::NulError)]
#[from_owned(std::str::Utf8Error)]
#[from_owned(tracing_subscriber::util::TryInitError)]
#[from_owned(image::ImageError)]
pub struct Error {
    pub status_code: StatusCode,
    pub open_subsonic_code: OpenSubsonicCode,
    pub source: Report,
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.source, f)
    }
}

impl From<Kind> for Error {
    fn from(source: Kind) -> Self {
        Self::new((&source).into(), (&source).into(), source)
    }
}

impl<T> From<Kind> for Result<T, Error> {
    fn from(value: Kind) -> Self {
        Err(value.into())
    }
}

impl Error {
    pub fn new(
        status_code: StatusCode,
        open_subsonic_code: OpenSubsonicCode,
        source: impl Into<color_eyre::Report>,
    ) -> Self {
        Self { status_code, open_subsonic_code, source: source.into() }
    }
}

impl From<Report> for Error {
    fn from(source: Report) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, OpenSubsonicCode::AGenericError, source)
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        let (status_code, open_subsonic_code) = match source.kind() {
            std::io::ErrorKind::NotFound => {
                (StatusCode::NOT_FOUND, OpenSubsonicCode::TheRequestedDataWasNotFound)
            }
            std::io::ErrorKind::PermissionDenied => {
                (StatusCode::FORBIDDEN, OpenSubsonicCode::UserIsNotAuthorizedForTheGivenOperation)
            }
            _ => return Report::from(source).into(),
        };
        Self::new(status_code, open_subsonic_code, source)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(source: diesel::result::Error) -> Self {
        let (status_code, open_subsonic_code) = match source {
            diesel::result::Error::NotFound => {
                (StatusCode::NOT_FOUND, OpenSubsonicCode::TheRequestedDataWasNotFound)
            }
            _ => return Report::from(source).into(),
        };
        Self::new(status_code, open_subsonic_code, source)
    }
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        if let Some(status) = source.status() {
            let (status_code, open_subsonic_code) = match status {
                StatusCode::NOT_FOUND => (status, OpenSubsonicCode::TheRequestedDataWasNotFound),
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    (status, OpenSubsonicCode::UserIsNotAuthorizedForTheGivenOperation)
                }
                _ => return Report::from(source).into(),
            };
            Self::new(status_code, open_subsonic_code, source)
        } else {
            Report::from(source).into()
        }
    }
}

impl From<rspotify::ClientError> for Error {
    fn from(source: rspotify::ClientError) -> Self {
        let (status_code, open_subsonic_code) = match source {
            rspotify::ClientError::Http(ref error) => match error.as_ref() {
                rspotify::http::HttpError::Client(error)
                    if let Some(status) = error.status()
                        && status == StatusCode::NOT_FOUND =>
                {
                    (StatusCode::NOT_FOUND, OpenSubsonicCode::TheRequestedDataWasNotFound)
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, OpenSubsonicCode::AGenericError),
            },
            rspotify::ClientError::Io(error) => return error.into(),
            rspotify::ClientError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                OpenSubsonicCode::UserIsNotAuthorizedForTheGivenOperation,
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, OpenSubsonicCode::AGenericError),
        };
        Self::new(status_code, open_subsonic_code, source)
    }
}

impl<T: Send + Sync + 'static> From<loole::SendError<T>> for Error {
    fn from(value: loole::SendError<T>) -> Self {
        Report::from(value).into()
    }
}

mod s3 {
    use ::s3::Error as S3Error;

    use super::*;

    impl From<S3Error> for Error {
        fn from(source: S3Error) -> Self {
            let (status_code, open_subsonic_code) = if let Some(status) = source.status()
                && status == StatusCode::NOT_FOUND
            {
                (StatusCode::NOT_FOUND, OpenSubsonicCode::TheRequestedDataWasNotFound)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, OpenSubsonicCode::AGenericError)
            };
            Self::new(status_code, open_subsonic_code, source)
        }
    }
}
