use std::ffi::OsString;

use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use loole::SendError;
use o2o::o2o;

#[derive(Debug, thiserror::Error, o2o)]
#[from_owned(diesel::result::Error| repeat(), return Self::Internal(@.into()))]
#[from_owned(std::io::Error)]
#[from_owned(isolang::ParseLanguageError)]
#[from_owned(uuid::Error)]
#[from_owned(lofty::error::LoftyError)]
#[from_owned(aws_sdk_s3::primitives::ByteStreamError)]
#[from_owned(std::num::TryFromIntError)]
#[from_owned(typed_path::StripPrefixError)]
#[from_owned(time::error::ComponentRange)]
#[from_owned(tokio::task::JoinError)]
#[from_owned(tokio::sync::AcquireError)]
#[from_owned(SdkError<GetObjectError>)]
#[from_owned(rsmpeg::error::RsmpegError)]
#[from_owned(std::ffi::NulError)]
#[from_owned(std::str::Utf8Error)]
#[from_owned(aws_sdk_s3::presigning::PresigningConfigError)]
#[from_owned(std::string::FromUtf8Error)]
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
    #[error("Language from database should not be null")]
    LanguageFromDatabaseIsNull,
    #[error("Inconsistency encountered while querying database for scan process")]
    DatabaseScanQueryInconsistent,
    #[error("Invalid config format for key {0}")]
    DatabaseInvalidConfigFormat(&'static str),

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

    #[error("Transcode output format is not supported")]
    TranscodeOutputFormatNotSupported,
    #[error("Encoder sample fmts are missing")]
    TranscodeEncoderSampleFmtsMissing,
    #[error("Could not get {0} av filter")]
    TranscodeAVFilterMissing(&'static str),
    #[error("Name is missing for sample format {0}")]
    TranscodeSampleFmtNameMissing(i32),

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

    #[error(transparent)]
    Internal(#[from] color_eyre::Report),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, status_message) = match &self {
            Error::InvalidParameter(_) | Error::SerializeRequest(_) | Error::InvalidRangeHeader => {
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

impl<T: Send + Sync + 'static> From<SendError<T>> for Error {
    fn from(value: SendError<T>) -> Self {
        Self::Internal(value.into())
    }
}
