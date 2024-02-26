use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::borrow::Cow;

use nghe_proc_macros::wrap_subsonic_response;

const BAD_REQUEST_MESSAGE: &str = "required parameter is missing";
const UNAUTHORIZED_MESSAGE: &str = "wrong username or password";
const FORBIDDEN_MESSAGE: &str = "user is not authorized for the given operation";
const NOT_FOUND_MESSAGE: &str = "the requested data was not found";

#[derive(Debug)]
pub enum OpenSubsonicError {
    Generic { source: anyhow::Error },
    BadRequest { message: Option<Cow<'static, str>> },
    Unauthorized { message: Option<Cow<'static, str>> },
    Forbidden { message: Option<Cow<'static, str>> },
    NotFound { message: Option<Cow<'static, str>> },
}

pub type OSResult<T> = Result<T, OpenSubsonicError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActualError {
    code: u8,
    message: Cow<'static, str>,
}

#[wrap_subsonic_response(success = false)]
#[derive(Debug)]
struct ErrorBody {
    error: ActualError,
}

fn error_to_json(code: u8, message: Cow<'static, str>) -> ErrorResponse {
    tracing::error!("{}", message);
    ErrorBody {
        error: ActualError { code, message },
    }
    .into()
}

impl<E> From<E> for OpenSubsonicError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        OpenSubsonicError::Generic { source: err.into() }
    }
}

impl OpenSubsonicError {
    pub fn into_cow_str(self) -> Cow<'static, str> {
        match self {
            OpenSubsonicError::Generic { source } => source.to_string().into(),
            OpenSubsonicError::BadRequest { message } => {
                message.unwrap_or(BAD_REQUEST_MESSAGE.into())
            }
            OpenSubsonicError::Unauthorized { message } => {
                message.unwrap_or(UNAUTHORIZED_MESSAGE.into())
            }
            OpenSubsonicError::Forbidden { message } => message.unwrap_or(FORBIDDEN_MESSAGE.into()),
            OpenSubsonicError::NotFound { message } => message.unwrap_or(NOT_FOUND_MESSAGE.into()),
        }
    }
}

impl IntoResponse for OpenSubsonicError {
    fn into_response(self) -> Response {
        error_to_json(
            match self {
                OpenSubsonicError::Generic { .. } => 0,
                OpenSubsonicError::BadRequest { .. } => 10,
                OpenSubsonicError::Unauthorized { .. } => 40,
                OpenSubsonicError::Forbidden { .. } => 50,
                OpenSubsonicError::NotFound { .. } => 70,
            },
            self.into_cow_str(),
        )
        .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::http::to_bytes;

    #[tokio::test]
    async fn test_generic_error() {
        let message = "error";
        let e: OpenSubsonicError = std::io::Error::new(std::io::ErrorKind::Other, message).into();
        assert_eq!(
            to_bytes(e.into_response()).await,
            to_bytes(error_to_json(0, message.into()).into_response()).await
        );
    }

    macro_rules! generate_custom_test {
        ($error_type:ident, $error_code:literal) => {
            paste::paste! {
              #[tokio::test]
              async fn [<test_ $error_type:snake _custom_message>]() {
                  let message = stringify!($error_type);
                  let e: OpenSubsonicError = OpenSubsonicError::$error_type { message: Some(message.into()) };
                  assert_eq!(
                    to_bytes(e.into_response()).await,
                    to_bytes(error_to_json($error_code, message.into()).into_response()).await
                  );
              }
            }
        };
    }

    macro_rules! generate_default_test {
        ($error_type:ident, $error_code:literal) => {
            paste::paste! {
              #[tokio::test]
              async fn [<test_ $error_type:snake _default_message>]() {
                  let e: OpenSubsonicError = OpenSubsonicError::$error_type { message: None };
                  assert_eq!(
                    to_bytes(e.into_response()).await,
                    to_bytes(error_to_json($error_code, [<$error_type:snake:upper _MESSAGE>].into()).into_response()).await
                  );
              }
            }
        };
    }

    generate_custom_test!(BadRequest, 10);
    generate_default_test!(BadRequest, 10);

    generate_custom_test!(Unauthorized, 40);
    generate_default_test!(Unauthorized, 40);

    generate_custom_test!(Forbidden, 50);
    generate_default_test!(Forbidden, 50);

    generate_custom_test!(NotFound, 70);
    generate_default_test!(NotFound, 70);
}
