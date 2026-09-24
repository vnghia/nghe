use axum::http::HeaderMap;
use axum_extra::headers::{self, HeaderMapExt};
use nghe_api::auth;
use uuid::Uuid;

use super::{Authentication, username};
use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

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

impl users::Authenticated {
    pub async fn from_headers(database: &Database, headers: &HeaderMap) -> Result<Self, Error> {
        if let Some(header) = headers.typed_get::<BearerAuthorization>() {
            header.authenticated(database).await
        } else if let Some(header) = headers.typed_get::<BaiscAuthorization>() {
            header.authenticated(database).await
        } else {
            error::Kind::MissingAuthenticationHeader.into()
        }
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
    async fn test_from_headers(
        #[future(awt)] mock: Mock,
        #[values(true, false)] ok: bool,
        #[values(true, false)] use_password: bool,
    ) {
        let user = mock.user(0).await;

        let mut http_request = http::Request::builder().body(()).unwrap();
        if use_password {
            let auth = user.auth_basic();
            http_request.headers_mut().typed_insert(BaiscAuthorization::basic(
                auth.username(),
                &if ok { auth.password().to_owned() } else { Password(16..32).fake::<String>() },
            ));
        } else {
            let auth = user.auth_bearer().await;
            http_request.headers_mut().typed_insert(if ok {
                auth
            } else {
                BearerAuthorization::bearer(&Faker.fake::<Uuid>().to_string()).unwrap()
            });
        }

        let authenticated =
            users::Authenticated::from_headers(mock.state(), http_request.headers()).await;
        assert_eq!(authenticated.is_ok(), ok);
        if ok {
            assert_eq!(authenticated.unwrap().id, user.id());
        }
    }
}
