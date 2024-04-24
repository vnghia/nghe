use serde::Deserialize;
use serde_repr::Deserialize_repr;
use strum::Display;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Deserialize_repr, Display, PartialEq, Eq)]
#[repr(u8)]
pub enum LastfmErrorCode {
    InvalidService = 2,
    InvalidMethod = 3,
    AuthenticationFailed = 4,
    InvalidFormat = 5,
    InvalidParameters = 6,
    InvalidResourceSpecified = 7,
    OperationFailed = 8,
    InvalidSessionKey = 9,
    InvalidApiKey = 10,
    ServiceOffline = 11,
    SubscribersOnly = 12,
    InvalidMethodSignatureSupplied = 13,
    UnauthorizedToken = 14,
    NotAvailableForStreaming = 15,
    ServiceUnavailable = 16,
    Login = 17,
    TrialExpired = 18,
    NotEnoughContent = 20,
    NotEnoughMembers = 21,
    NotEnoughFans = 22,
    NotEnoughNeighbours = 23,
    NoPeakRadio = 24,
    RadioNotFound = 25,
    ApiKeySuspended = 26,
    Deprecated = 27,
    RateLimitExceded = 28,
}

#[derive(Debug, Deserialize, Error)]
#[error("Lastfm error {code}: {message}")]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct LastfmError {
    pub code: LastfmErrorCode,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    LastFm(#[from] LastfmError),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    SerQuery(#[from] serde_html_form::ser::Error),
}
