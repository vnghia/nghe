#![allow(unused_variables)]

use std::fmt::Debug;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use color_eyre::Report;
use o2o::o2o;

use crate::file::audio;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum OpensubsonicCode {
    AGenericError = 0,
    RequiredParameterIsMissing = 10,
    WrongUsernameOrPassword = 40,
    InvalidApiKey = 44,
    UserIsNotAuthorizedForTheGivenOperation = 50,
    TheRequestedDataWasNotFound = 70,
}

#[derive(Debug, thiserror::Error, o2o)]
#[ref_into(StatusCode)]
#[ref_into(OpensubsonicCode)]
pub enum Kind {
    // Request error
    #[error(transparent)]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    DeserializeBytes(#[from] axum::extract::rejection::BytesRejection),
    #[error(transparent)]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    ExtractForm(#[from] axum::extract::rejection::RawFormRejection),
    #[error(transparent)]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    DeserializeForm(#[from] serde_html_form::de::Error),

    #[error("Missing authentication header")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    MissingAuthenticationHeader,
    #[error("Invalid bearer authorization format")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    InvalidBearerAuthorizationFormat,
    #[error("Wrong username or password")]
    #[into(StatusCode| StatusCode::UNAUTHORIZED)]
    #[into(OpensubsonicCode| OpensubsonicCode::WrongUsernameOrPassword)]
    WrongUsernameOrPassword,
    #[error("Invalid API key")]
    #[into(StatusCode| StatusCode::UNAUTHORIZED)]
    #[into(OpensubsonicCode| OpensubsonicCode::InvalidApiKey)]
    InvalidApiKey,
    #[error("User is not authorized for the given operation")]
    #[into(StatusCode| StatusCode::FORBIDDEN)]
    #[into(OpensubsonicCode| OpensubsonicCode::UserIsNotAuthorizedForTheGivenOperation)]
    Forbidden,

    #[error("Invalid range header {0:?}")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    InvalidRangeHeader(axum_extra::headers::Range),

    #[error("Found more time than id in scrobble artist")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    InvalidScrobbleTimeSize,

    // Database error
    #[error("Could not decrypt database value")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    DatabaseValueDecryptionFailed,
    #[error("Invalid database config format for key {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidDatabaseConfigFomat(&'static str),
    #[error("Database corruption detected")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    DatabaseCorruptionDetected,

    // Filesystem error
    #[error("Missing extension in path {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingPathExtension(typed_path::Utf8TypedPathBuf),
    #[error("Missing parent in path {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingPathParent(typed_path::Utf8TypedPathBuf),
    #[error("Path {0} does not have correct encoding")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidTypedPathPlatform(typed_path::Utf8TypedPathBuf),
    #[error("Path {0} is not an absolute path")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidAbsolutePath(typed_path::Utf8TypedPathBuf),
    #[error("Path {0} is not a directory path")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidDirectoryPath(typed_path::Utf8TypedPathBuf),
    #[error("Non UTF-8 path encountered: {0:?}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    NonUTF8PathEncountered(std::ffi::OsString),

    #[error("Missing size in file/object metadata")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingFileSize,
    #[error("Empty file encountered")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    EmptyFileEncountered,

    // Media error
    #[error("Could not found vorbis comments in format {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingVorbisComments(audio::Format),
    #[error("Could not found id3v2 tag in format {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingId3V2Tag(audio::Format),

    #[error("Missing media name")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingMediaName,
    #[error("Missing song artist name")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingSongArtistName,
    #[error("Invalid artist name format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidArtistNameFormat,
    #[error("Found more musicbrainz id than artist name")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidMbzIdSize,

    #[error("Invalid date tag format with value {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidDateTagFormat(String),
    #[error("Invalid musicbrainz id tag format with value {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidMbzIdTagFormat(String),
    #[error(transparent)]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidLanguageTagFormat(#[from] isolang::ParseLanguageError),
    #[error(
        "Could not parse position from track number {track_number:?}, track total \
         {track_total:?}, disc number {disc_number:?} and disc total {disc_total:?}"
    )]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidPositionTagFormat {
        track_number: Option<String>,
        track_total: Option<String>,
        disc_number: Option<String>,
        disc_total: Option<String>,
    },

    #[error("Invalid id3v2 frame id config format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidId3v2FrameIdConfigFormat,
    #[error("Invalid id3v2 frame id config type")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidId3v2FrameIdConfigType,

    // Image error
    #[error("Missing image format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingImageFormat,
    #[error("Unsupported image format {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    UnsupportedImageFormat(String),

    #[error("Missing cover art directory config")]
    #[into(StatusCode| StatusCode::NOT_FOUND)]
    #[into(OpensubsonicCode| OpensubsonicCode::TheRequestedDataWasNotFound)]
    MissingCoverArtDirectoryConfig,

    // Lyrics error
    #[error("Could not parse lyrics from {0}")]
    #[into(StatusCode| StatusCode::NOT_FOUND)]
    #[into(OpensubsonicCode| OpensubsonicCode::TheRequestedDataWasNotFound)]
    InvalidLyricsLrcFormat(String),

    // Rspotify error
    #[error("Invalid spotify id format with value {0}")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    InvalidSpotifyIdFormat(String),

    // LastFM Error
    #[error("Could not build LastFM request URL")]
    #[into(StatusCode| StatusCode::BAD_REQUEST)]
    #[into(OpensubsonicCode| OpensubsonicCode::RequiredParameterIsMissing)]
    BuildLastFMRequestURLFailed,

    // Transcode error
    #[error("No audio track found in media")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingAudioTrack,
    #[error("Missing encoder")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingEncoder,
    #[error("Missing sample fmts for encoder")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingEncoderSampleFmts,
    #[error("Missing av filter with name {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingAVFilter(&'static str),
    #[error("Missing sample fmt name for fmt id {0}")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    MissingSampleFmtName(i32),

    // Various error
    #[error("Invalid index ignore prefixes format")]
    #[into(StatusCode| StatusCode::INTERNAL_SERVER_ERROR)]
    #[into(OpensubsonicCode| OpensubsonicCode::AGenericError)]
    InvalidIndexIgnorePrefixesFormat,

    #[error("The requested data was not found")]
    #[into(StatusCode| StatusCode::NOT_FOUND)]
    #[into(OpensubsonicCode| OpensubsonicCode::TheRequestedDataWasNotFound)]
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
#[from_owned(lofty::error::LoftyError)]
#[from_owned(reqwest::header::ToStrError)]
#[from_owned(typed_path::StripPrefixError)]
#[from_owned(aws_sdk_s3::primitives::ByteStreamError)]
#[from_owned(aws_sdk_s3::presigning::PresigningConfigError)]
#[from_owned(tokio::task::JoinError)]
#[from_owned(rsmpeg::error::RsmpegError)]
#[from_owned(tokio::sync::AcquireError)]
#[from_owned(std::ffi::NulError)]
#[from_owned(std::str::Utf8Error)]
#[from_owned(tracing_subscriber::util::TryInitError)]
#[from_owned(image::ImageError)]
pub struct Error {
    pub status_code: StatusCode,
    pub opensubsonic_code: OpensubsonicCode,
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
        opensubsonic_code: OpensubsonicCode,
        source: impl Into<color_eyre::Report>,
    ) -> Self {
        Self { status_code, opensubsonic_code, source: source.into() }
    }
}

impl From<Report> for Error {
    fn from(source: Report) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, OpensubsonicCode::AGenericError, source)
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        let (status_code, opensubsonic_code) = match source.kind() {
            std::io::ErrorKind::NotFound => {
                (StatusCode::NOT_FOUND, OpensubsonicCode::TheRequestedDataWasNotFound)
            }
            std::io::ErrorKind::PermissionDenied => {
                (StatusCode::FORBIDDEN, OpensubsonicCode::UserIsNotAuthorizedForTheGivenOperation)
            }
            _ => return Report::from(source).into(),
        };
        Self::new(status_code, opensubsonic_code, source)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(source: diesel::result::Error) -> Self {
        let (status_code, opensubsonic_code) = match source {
            diesel::result::Error::NotFound => {
                (StatusCode::NOT_FOUND, OpensubsonicCode::TheRequestedDataWasNotFound)
            }
            _ => return Report::from(source).into(),
        };
        Self::new(status_code, opensubsonic_code, source)
    }
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        if let Some(status) = source.status() {
            let (status_code, opensubsonic_code) = match status {
                StatusCode::NOT_FOUND => (status, OpensubsonicCode::TheRequestedDataWasNotFound),
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                    (status, OpensubsonicCode::UserIsNotAuthorizedForTheGivenOperation)
                }
                _ => return Report::from(source).into(),
            };
            Self::new(status_code, opensubsonic_code, source)
        } else {
            Report::from(source).into()
        }
    }
}

impl From<rspotify::ClientError> for Error {
    fn from(source: rspotify::ClientError) -> Self {
        let (status_code, opensubsonic_code) = match source {
            rspotify::ClientError::Http(ref error) => match error.as_ref() {
                rspotify::http::HttpError::Client(error)
                    if let Some(status) = error.status()
                        && status == StatusCode::NOT_FOUND =>
                {
                    (StatusCode::NOT_FOUND, OpensubsonicCode::TheRequestedDataWasNotFound)
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, OpensubsonicCode::AGenericError),
            },
            rspotify::ClientError::Io(error) => return error.into(),
            rspotify::ClientError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                OpensubsonicCode::UserIsNotAuthorizedForTheGivenOperation,
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, OpensubsonicCode::AGenericError),
        };
        Self::new(status_code, opensubsonic_code, source)
    }
}

impl<T: Send + Sync + 'static> From<loole::SendError<T>> for Error {
    fn from(value: loole::SendError<T>) -> Self {
        Report::from(value).into()
    }
}

mod aws {
    use aws_sdk_s3::error::SdkError;
    use aws_sdk_s3::operation::get_object::GetObjectError;
    use aws_sdk_s3::operation::head_object::HeadObjectError;
    use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;

    use super::*;

    impl From<SdkError<ListObjectsV2Error>> for Error {
        fn from(source: SdkError<ListObjectsV2Error>) -> Self {
            let (status_code, opensubsonic_code) = if let SdkError::ServiceError(ref error) = source
                && let ListObjectsV2Error::NoSuchBucket(_) = error.err()
            {
                (StatusCode::NOT_FOUND, OpensubsonicCode::TheRequestedDataWasNotFound)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, OpensubsonicCode::AGenericError)
            };
            Self::new(status_code, opensubsonic_code, source)
        }
    }

    impl From<SdkError<GetObjectError>> for Error {
        fn from(source: SdkError<GetObjectError>) -> Self {
            let (status_code, opensubsonic_code) = if let SdkError::ServiceError(ref error) = source
                && let GetObjectError::NoSuchKey(_) = error.err()
            {
                (StatusCode::NOT_FOUND, OpensubsonicCode::TheRequestedDataWasNotFound)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, OpensubsonicCode::AGenericError)
            };
            Self::new(status_code, opensubsonic_code, source)
        }
    }

    impl From<SdkError<HeadObjectError>> for Error {
        fn from(source: SdkError<HeadObjectError>) -> Self {
            let (status_code, opensubsonic_code) = if let SdkError::ServiceError(ref error) = source
                && let HeadObjectError::NotFound(_) = error.err()
            {
                (StatusCode::NOT_FOUND, OpensubsonicCode::TheRequestedDataWasNotFound)
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, OpensubsonicCode::AGenericError)
            };
            Self::new(status_code, opensubsonic_code, source)
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status_code, self.source.to_string()).into_response()
    }
}
