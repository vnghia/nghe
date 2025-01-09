use axum::extract::{FromRef, FromRequest, Request};
use nghe_api::auth;
use nghe_api::auth::form::Trait;
use nghe_api::common::FormRequest;
use uuid::Uuid;

use super::{Authentication, Authorization};
use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

pub struct Form<R> {
    pub id: Uuid,
    pub request: R,
}

impl Authentication for auth::Form<'_, '_, '_, '_> {
    async fn authenticated(&self, database: &Database) -> Result<users::Authenticated, Error> {
        match self {
            auth::Form::Username(username) => username.authenticated(database).await,
            auth::Form::ApiKey(api_key) => todo!(),
        }
    }
}

impl<S, R> FromRequest<S> for Form<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: for<'form> FormRequest<'form, 'form, 'form, 'form, 'form> + Authorization + Send,
{
    type Rejection = Error;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::RawForm(bytes) =
            axum::extract::RawForm::from_request(request, &()).await.map_err(error::Kind::from)?;
        let form: R::AuthForm = serde_html_form::from_bytes(&bytes).map_err(error::Kind::from)?;
        let id = form.auth().login::<S, R>(state).await?.id;
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
    use fake::{Fake, Faker};
    use nghe_proc_macro::api_derive;
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_from_request(
        #[future(awt)] mock: Mock,
        #[values(true, false)] get: bool,
        #[values(true, false)] ok: bool,
        #[values(Some(true), Some(false))] use_token: Option<bool>,
    ) {
        #[api_derive(fake = true)]
        #[endpoint(path = "test", url_only = true, same_crate = false)]
        #[derive(Clone, Copy, PartialEq)]
        struct Request {
            param_one: i32,
            param_two: u32,
        }

        impl Authorization for Request {
            fn authorized(_: crate::orm::users::Role) -> bool {
                true
            }
        }

        let request: Request = Faker.fake();
        let user = mock.user(0).await;
        let auth = user.auth_form(use_token);
        let auth = if ok {
            user.auth_form(use_token)
        } else {
            match auth {
                auth::Form::Username(_) => auth::Form::Username(Faker.fake()),
                auth::Form::ApiKey(_) => auth::Form::ApiKey(Faker.fake()),
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
