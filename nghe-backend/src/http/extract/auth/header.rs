use std::marker::PhantomData;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::headers::{self, HeaderMapExt};

use super::{login, AuthN, AuthZ};
use crate::database::Database;
use crate::Error;

pub struct Header<R> {
    _request: PhantomData<R>,
}

pub type BaiscAuthorization = headers::Authorization<headers::authorization::Basic>;

impl AuthN for BaiscAuthorization {
    fn username(&self) -> &str {
        self.username()
    }

    fn is_authenticated(&self, password: impl AsRef<[u8]>) -> bool {
        self.password().as_bytes() == password.as_ref()
    }
}

impl<S, R> FromRequestParts<S> for Header<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: AuthZ,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .typed_get::<BaiscAuthorization>()
            .ok_or_else(|| Error::MissingAuthenticationHeader)?;
        login::<R, _>(state, &header).await?;
        Ok(Self { _request: PhantomData })
    }
}

#[cfg(test)]
mod tests {
    use axum::http;
    use axum_extra::headers::HeaderMapExt;
    use fake::faker::internet::en::{Password, Username};
    use fake::Fake;
    use rstest::rstest;

    use super::*;
    use crate::test::{mock, Mock};

    #[rstest]
    fn test_is_authenticated(#[values(true, false)] ok: bool) {
        let username = Username().fake::<String>();
        let password = Password(16..32).fake::<String>();
        let header = BaiscAuthorization::basic(
            &username,
            &if ok { password.clone() } else { Password(16..32).fake::<String>() },
        );
        assert_eq!(header.is_authenticated(&password), ok);
    }

    #[rstest]
    #[tokio::test]
    async fn test_from_request_parts(#[future(awt)] mock: Mock, #[values(true, false)] ok: bool) {
        struct Request;

        impl AuthZ for Request {
            fn is_authorized(_: crate::orm::users::Role) -> bool {
                true
            }
        }

        let user = mock.user(0).await;
        let auth = user.auth_header();

        let mut http_request = http::Request::builder().body(()).unwrap();
        http_request.headers_mut().typed_insert(BaiscAuthorization::basic(
            auth.username(),
            &if ok { auth.password().to_owned() } else { Password(16..32).fake::<String>() },
        ));
        let mut parts = http_request.into_parts().0;

        let header = Header::<Request>::from_request_parts(&mut parts, mock.state()).await;
        assert_eq!(header.is_ok(), ok);
    }
}
