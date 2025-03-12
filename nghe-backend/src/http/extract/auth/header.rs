use std::marker::PhantomData;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::headers::{self, HeaderMapExt};
use nghe_api::auth;
use uuid::Uuid;

use super::{Authentication, username};
use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

#[derive(Debug)]
pub struct Header<R> {
    _request: PhantomData<R>,
    pub user: users::Authenticated,
}

pub type BearerAuthorization = headers::Authorization<headers::authorization::Bearer>;
pub type BaiscAuthorization = headers::Authorization<headers::authorization::Basic>;

impl Authentication for BearerAuthorization {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error> {
        auth::ApiKey::from(
            self.token()
                .parse::<Uuid>()
                .map_err(|_| error::Kind::InvalidBearerAuthorizationFormat)?,
        )
        .authenticated(database)
        .await
    }
}

impl username::Authentication for BaiscAuthorization {
    fn username(&self) -> &str {
        self.username()
    }

    fn authenticated(&self, password: impl AsRef<[u8]>) -> bool {
        self.password().as_bytes() == password.as_ref()
    }
}

impl<S, R> FromRequestParts<S> for Header<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: Send,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let database = &Database::from_ref(state);
        let user = if let Some(header) = parts.headers.typed_get::<BearerAuthorization>() {
            header.authenticated(database).await?
        } else if let Some(header) = parts.headers.typed_get::<BaiscAuthorization>() {
            header.authenticated(database).await?
        } else {
            return error::Kind::MissingAuthenticationHeader.into();
        };
        Ok(Self { _request: PhantomData, user })
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use axum::http;
    use axum_extra::headers::HeaderMapExt;
    use fake::faker::internet::en::{Password, Username};
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::username::Authentication;
    use super::*;
    use crate::test::{Mock, mock};

    struct Request;

    #[rstest]
    fn test_authenticated(#[values(true, false)] ok: bool) {
        let username = Username().fake::<String>();
        let password = Password(16..32).fake::<String>();
        let header = BaiscAuthorization::basic(
            &username,
            &if ok { password.clone() } else { Password(16..32).fake::<String>() },
        );
        assert_eq!(header.authenticated(&password), ok);
    }

    #[rstest]
    #[tokio::test]
    async fn test_from_request_parts_bearer(
        #[future(awt)] mock: Mock,
        #[values(true, false)] ok: bool,
    ) {
        let user = mock.user(0).await;
        let auth = user.auth_bearer().await;

        let mut http_request = http::Request::builder().body(()).unwrap();
        http_request.headers_mut().typed_insert(if ok {
            auth
        } else {
            BearerAuthorization::bearer(&Faker.fake::<Uuid>().to_string()).unwrap()
        });
        let mut parts = http_request.into_parts().0;

        let header = Header::<Request>::from_request_parts(&mut parts, mock.state()).await;
        assert_eq!(header.is_ok(), ok);
        if ok {
            assert_eq!(header.unwrap().user.id, user.id());
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_from_request_parts_basic(
        #[future(awt)] mock: Mock,
        #[values(true, false)] ok: bool,
    ) {
        let user = mock.user(0).await;
        let auth = user.auth_basic();

        let mut http_request = http::Request::builder().body(()).unwrap();
        http_request.headers_mut().typed_insert(BaiscAuthorization::basic(
            auth.username(),
            &if ok { auth.password().to_owned() } else { Password(16..32).fake::<String>() },
        ));
        let mut parts = http_request.into_parts().0;

        let header = Header::<Request>::from_request_parts(&mut parts, mock.state()).await;
        assert_eq!(header.is_ok(), ok);
        if ok {
            assert_eq!(header.unwrap().user.id, user.id());
        }
    }
}
