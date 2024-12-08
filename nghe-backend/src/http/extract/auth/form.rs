use axum::extract::{FromRef, FromRequest, Request};
use nghe_api::auth;
use nghe_api::auth::form::Trait;
use nghe_api::common::FormRequest;
use uuid::Uuid;

use super::{login, AuthN, AuthZ};
use crate::database::Database;
use crate::{error, Error};

pub struct Form<R> {
    pub id: Uuid,
    pub request: R,
}

impl AuthN for auth::Form<'_, '_> {
    fn username(&self) -> &str {
        match self {
            auth::Form::Token(auth) => auth.username.as_ref(),
        }
    }

    fn is_authenticated(&self, password: impl AsRef<[u8]>) -> bool {
        match self {
            auth::Form::Token(auth) => {
                let password = password.as_ref();
                let password_token = auth::Token::new(password, auth.salt.as_bytes());
                password_token == auth.token
            }
        }
    }
}

impl<S, R> FromRequest<S> for Form<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: for<'u, 's> FormRequest<'u, 's> + AuthZ + Send,
{
    type Rejection = Error;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let form: R::AuthForm = axum_extra::extract::Form::from_request(request, &())
            .await
            .map_err(error::Kind::from)?
            .0;
        let id = login::<R, _>(state, form.auth()).await?;
        Ok(Self { id, request: form.request() })
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    #![allow(unexpected_cfgs)]

    use axum::body::Body;
    use axum::http;
    use axum_extra::headers::{self, HeaderMapExt};
    use concat_string::concat_string;
    use fake::faker::internet::en::{Password, Username};
    use fake::{Fake, Faker};
    use nghe_proc_macro::api_derive;
    use rstest::rstest;

    use super::*;
    use crate::test::{mock, Mock};

    #[rstest]
    fn test_is_authenticated(#[values(true, false)] ok: bool) {
        let password = Password(16..32).fake::<String>().into_bytes();
        let salt = Password(8..16).fake::<String>();
        let token = auth::Token::new(&password, &salt);

        let salt = if ok { (&salt).into() } else { Password(8..16).fake::<String>().into() };

        let form: auth::Form =
            auth::token::Auth { username: Username().fake::<String>().into(), salt, token }.into();
        assert_eq!(form.is_authenticated(password), ok);
    }

    #[rstest]
    #[tokio::test]
    async fn test_from_request(
        #[future(awt)] mock: Mock,
        #[values(true, false)] get: bool,
        #[values(true, false)] ok: bool,
    ) {
        #[api_derive(fake = true)]
        #[endpoint(path = "test", url_only = true, same_crate = false)]
        #[derive(Clone, Copy, PartialEq)]
        struct Request {
            param_one: i32,
            param_two: u32,
        }

        impl AuthZ for Request {
            fn is_authorized(_: crate::orm::users::Role) -> bool {
                true
            }
        }

        let request: Request = Faker.fake();
        let user = mock.user(0).await;
        let auth = user.auth_form();

        let auth = if ok {
            auth
        } else {
            match auth {
                auth::Form::Token(auth) => {
                    auth::token::Auth { salt: Faker.fake::<String>().into(), ..auth }.into()
                }
            }
        };

        let builder = http::Request::builder();
        let query =
            serde_html_form::to_string(<Request as FormRequest>::AuthForm::new(request, auth))
                .unwrap();
        let http_request = if get {
            builder
                .method(http::Method::GET)
                .uri(concat_string!("/test?", query))
                .body(Body::empty())
                .unwrap()
        } else {
            let mut http_request =
                builder.method(http::Method::POST).uri("/test").body(Body::from(query)).unwrap();
            http_request.headers_mut().typed_insert(headers::ContentType::form_url_encoded());
            http_request
        };

        let form_request = Form::<Request>::from_request(http_request, mock.state()).await;

        if ok {
            let form_request = form_request.unwrap();
            assert_eq!(form_request.id, user.id());
            assert_eq!(form_request.request, request);
        } else {
            assert!(form_request.is_err());
        }
    }
}
