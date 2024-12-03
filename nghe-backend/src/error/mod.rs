use std::ffi::OsString;

use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use loole::SendError;
use o2o::o2o;

#[derive(Debug, thiserror::Error, o2o)]
#[from_owned(std::io::Error| repeat(), return Self::Internal(@.into()))]
#[from_owned(isolang::ParseLanguageError)]
#[from_owned(uuid::Error)]
#[from_owned(lofty::error::LoftyError)]
#[from_owned(aws_sdk_s3::primitives::ByteStreamError)]
#[from_owned(std::num::TryFromIntError)]
#[from_owned(typed_path::StripPrefixError)]
#[from_owned(time::error::ComponentRange)]
#[from_owned(time::error::ConversionRange)]
#[from_owned(tokio::task::JoinError)]
#[from_owned(tokio::sync::AcquireError)]
#[from_owned(SdkError<GetObjectError>)]
#[from_owned(SdkError<HeadObjectError>)]
#[from_owned(rsmpeg::error::RsmpegError)]
#[from_owned(std::ffi::NulError)]
#[from_owned(std::str::Utf8Error)]
#[from_owned(aws_sdk_s3::presigning::PresigningConfigError)]
#[from_owned(std::string::FromUtf8Error)]
#[from_owned(axum::extract::rejection::StringRejection)]
#[from_owned(strum::ParseError)]
#[from_owned(bitcode::Error)]
#[from_owned(rspotify::ClientError)]
#[from_owned(rspotify::model::IdError)]
#[from_owned(reqwest::Error)]
#[from_owned(reqwest::header::ToStrError)]
pub enum Error {
    #[error("{0}")]
    InvalidParameter(&'static str),
    #[error("Could not serialize get request with no query parameters")]
    GetRequestMissingQueryParameters,
    #[error("Could not serialize auth parameters with query {0}")]
    SerializeAuthParameters(String),
    #[error("Could not serialize request parameters with query {0}")]
    SerializeRequestParameters(String),
    #[error("Could not serialize binary request")]
    SerializeBinaryRequest,
    #[error("Could not serialize json request due to {0}")]
    SerializeJsonRequest(String),
    #[error("Could not find authentication header")]
    MissingAuthenticationHeader,
    #[error(transparent)]
    ExtractRequestBody(#[from] axum::extract::rejection::BytesRejection),
    #[error("Scrobble request must have more id than time")]
    ScrobbleRequestMustHaveBeMoreIdThanTime,

    #[error("Resource not found")]
    NotFound,

    #[error("Could not checkout a connection from connection pool")]
    CheckoutConnectionPool,
    #[error("Could not decrypt value from database")]
    DecryptDatabaseValue,
    #[error("Inconsistency encountered while querying database for scan process")]
    DatabaseScanQueryInconsistent,
    #[error("Invalid config format for key {0}")]
    DatabaseInvalidConfigFormat(&'static str),
    #[error("Song duration is empty")]
    DatabaseSongDurationIsEmpty,

    #[error("{0}")]
    Unauthorized(&'static str),
    #[error("Could not login due to bad credentials")]
    Unauthenticated,
    #[error("You need to have {0} role to perform this action")]
    MissingRole(&'static str),
    #[error("Range header is invalid")]
    InvalidRangeHeader,

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
    #[error("Artist name should not be empty")]
    MediaArtistNameEmpty,
    #[error("Could not read vorbis comments from flac file")]
    MediaFlacMissingVorbisComments,
    #[error("Could not find audio track in the media file")]
    MediaAudioTrackMissing,
    #[error("Media picture format is missing")]
    MediaPictureMissingFormat,
    #[error("Media picture format {0} is unsupported")]
    MediaPictureUnsupportedFormat(String),
    #[error("Media cover art dir is not enabled")]
    MediaCoverArtDirIsNotEnabled,

    #[error("Transcode output format is not supported")]
    TranscodeOutputFormatNotSupported,
    #[error("Encoder sample fmts are missing")]
    TranscodeEncoderSampleFmtsMissing,
    #[error("Could not get {0} av filter")]
    TranscodeAVFilterMissing(&'static str),
    #[error("Name is missing for sample format {0}")]
    TranscodeSampleFmtNameMissing(i32),

    #[error("Path extension is missing")]
    PathExtensionMissing,
    #[error("Absolute file path does not have parent directory")]
    AbsoluteFilePathDoesNotHaveParentDirectory,
    #[error("S3 path is not an absolute unix path: {0}")]
    FilesystemS3InvalidPath(String),
    #[error("S3 object does not have size information")]
    FilesystemS3MissingObjectSize,
    #[error("Non UTF-8 path encountered: {0:?}")]
    FilesystemLocalNonUTF8PathEncountered(OsString),
    #[error("Typed path has wrong platform information")]
    FilesystemTypedPathWrongPlatform,

    #[error("Prefix does not end with whitespace")]
    ConfigIndexIgnorePrefixEndWithoutSpace,

    #[error("Could not convert float to integer with value {0}")]
    CouldNotConvertFloatToInteger(f32),

    #[error(transparent)]
    Internal(#[from] color_eyre::Report),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, status_message) = match &self {
            Error::InvalidParameter(_)
            | Error::GetRequestMissingQueryParameters
            | Error::SerializeAuthParameters(_)
            | Error::SerializeRequestParameters(_)
            | Error::SerializeBinaryRequest
            | Error::SerializeJsonRequest(_)
            | Error::MissingAuthenticationHeader
            | Error::InvalidRangeHeader => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::ExtractRequestBody(_) => {
                (StatusCode::BAD_REQUEST, "Could not extract request body".into())
            }
            Error::Unauthenticated => (StatusCode::FORBIDDEN, self.to_string()),
            Error::Unauthorized(_) | Error::MissingRole(_) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            Error::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into()),
        };
        (status_code, status_message).into_response()
    }
}

impl<T: Send + Sync + 'static> From<SendError<T>> for Error {
    fn from(value: SendError<T>) -> Self {
        Self::Internal(value.into())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Error::NotFound,
            _ => Error::Internal(value.into()),
        }
    }
}
