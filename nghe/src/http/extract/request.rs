use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest};
use axum::http::Method;
use axum_extra::headers::{self, HeaderMapExt};

use crate::database::Database;
use crate::error;
use crate::http::serializable::ErrorResponse;
use crate::orm::users;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Type {
    Form,
    Json,
}

#[derive(Debug)]
pub struct Validated<R> {
    pub ty: Type,
    pub request: R,
}

#[derive(Debug)]
pub struct Authenticated<R> {
    pub validated: Validated<R>,
    pub user: users::Authenticated,
}

impl<S, R> FromRequest<S> for Validated<R>
where
    S: Send + Sync,
    R: for<'form> nghe_api::http::Request<'form, 'form, 'form, 'form, 'form> + Send,
{
    type Rejection = ErrorResponse;

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip_all, err(Debug)))]
    async fn from_request(
        request: axum::extract::Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let method = request.method();

        if method == Method::GET {
            if let Some(query) = request.uri().query() {
                Self::from_form(query.as_bytes()).map_err(|error| (Type::Form, error).into())
            } else {
                Ok(Self {
                    ty: Type::Json,
                    request: R::UNIT.ok_or_else(|| error::Kind::MissingRequestBody)?,
                })
            }
        } else if method == Method::POST {
            let headers = request.headers();
            let content_type = headers
                .typed_get::<headers::ContentType>()
                .ok_or_else(|| error::Kind::MissingContentTypeHeader)?;

            if content_type == headers::ContentType::form_url_encoded() {
                let body = Bytes::from_request(request, state)
                    .await
                    .map_err(|error| (Type::Form, error))?;
                Self::from_form(&body).map_err(|error| (Type::Form, error).into())
            } else {
                let body = Bytes::from_request(request, state).await.map_err(error::Kind::from)?;
                if content_type == headers::ContentType::json() {
                    Ok(Self {
                        ty: Type::Json,
                        request: serde_json::from_slice(&body)
                            .map_err(|error| (Type::Json, error))?,
                    })
                } else {
                    ErrorResponse::from(error::Kind::UnsupportedContentType(content_type)).into()
                }
            }
        } else {
            ErrorResponse::from(error::Kind::MethodNotAllowed(method.to_owned())).into()
        }
    }
}

impl<S, R> FromRequest<S> for Authenticated<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: for<'form> nghe_api::http::Request<'form, 'form, 'form, 'form, 'form> + Send,
{
    type Rejection = ErrorResponse;

    #[cfg_attr(not(coverage_nightly), tracing::instrument(skip_all, err(Debug)))]
    async fn from_request(
        request: axum::extract::Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let database = Database::from_ref(state);
        let method = request.method();

        if method == Method::GET {
            if let Some(query) = request.uri().query() {
                Self::from_form(&database, query.as_bytes())
                    .await
                    .map_err(|error| (Type::Form, error).into())
            } else {
                Ok(Self {
                    validated: Validated {
                        ty: Type::Json,
                        request: R::UNIT.ok_or_else(|| error::Kind::MissingRequestBody)?,
                    },
                    user: users::Authenticated::from_headers(&database, request.headers())
                        .await
                        .map_err(|error| (Type::Json, error))?,
                })
            }
        } else if method == Method::POST {
            let headers = request.headers();
            let content_type = headers
                .typed_get::<headers::ContentType>()
                .ok_or_else(|| error::Kind::MissingContentTypeHeader)?;

            if content_type == headers::ContentType::form_url_encoded() {
                let body = Bytes::from_request(request, state).await.map_err(error::Kind::from)?;
                Self::from_form(&database, &body).await.map_err(|error| (Type::Form, error).into())
            } else {
                let user = users::Authenticated::from_headers(&database, headers)
                    .await
                    .map_err(|error| (Type::Json, error))?;
                let body = Bytes::from_request(request, state).await.map_err(error::Kind::from)?;
                if content_type == headers::ContentType::json() {
                    Ok(Self {
                        validated: Validated {
                            ty: Type::Json,
                            request: serde_json::from_slice(&body).map_err(error::Kind::from)?,
                        },
                        user,
                    })
                } else {
                    ErrorResponse::from(error::Kind::UnsupportedContentType(content_type)).into()
                }
            }
        } else {
            ErrorResponse::from(error::Kind::MethodNotAllowed(method.to_owned())).into()
        }
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
    use fake::faker::internet::en::Password;
    use fake::{Fake, Faker};
    use nghe_api::auth;
    use nghe_api::auth::form::Trait as _;
    use nghe_proc_macro::api_derive;
    use rstest::rstest;
    use uuid::Uuid;

    use super::super::auth::header::{BasicAuthorization, BearerAuthorization};
    use super::*;
    use crate::test::{Mock, mock};

    #[api_derive(fake = true)]
    #[endpoint(path = "test", url_only = true, api_crate = nghe_api)]
    #[derive(Clone, Copy, PartialEq)]
    struct Request {
        param_one: i32,
        param_two: u32,
    }

    #[rstest]
    #[tokio::test]
    async fn test_from_request_form(
        #[future(awt)] mock: Mock,
        #[values(true, false)] get: bool,
        #[values(true, false)] auth: bool,
        #[values(true, false)] ok: bool,
        #[values(None, Some(true), Some(false))] use_token: Option<bool>,
    ) {
        let body: Request = Faker.fake();
        let user = mock.user(0).await;

        let builder = http::Request::builder();
        let query = if auth {
            let auth = user.auth_form(use_token).await;
            let auth = if ok {
                auth
            } else {
                match auth {
                    auth::Form::Username(_) => auth::Form::Username(Faker.fake()),
                    auth::Form::ApiKey(_) => auth::Form::ApiKey(Faker.fake()),
                }
            };

            serde_html_form::to_string(<Request as nghe_api::http::Request>::AuthForm::new(
                body, auth,
            ))
            .unwrap()
        } else {
            serde_html_form::to_string(body).unwrap()
        };

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

        if auth {
            let request = Authenticated::<Request>::from_request(http_request, mock.state()).await;

            if ok {
                let request = request.unwrap();
                assert_eq!(request.user.id, user.id());
                assert_eq!(request.validated.request, body);
                assert_eq!(request.validated.ty, Type::Form);
            } else {
                let error = request.unwrap_err();
                assert_eq!(error.ty, Type::Form);
            }
        } else {
            let request =
                Validated::<Request>::from_request(http_request, mock.state()).await.unwrap();
            assert_eq!(request.request, body);
            assert_eq!(request.ty, Type::Form);
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_from_request_json(
        #[future(awt)] mock: Mock,
        #[values(true, false)] auth: bool,
        #[values(true, false)] ok: bool,
        #[values(true, false)] use_password: bool,
    ) {
        let body: Request = Faker.fake();
        let user = mock.user(0).await;

        let mut http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(axum::body::Body::new(serde_json::to_string(&body).unwrap()))
            .unwrap();
        http_request.headers_mut().typed_insert(headers::ContentType::json());
        if auth {
            if use_password {
                let auth = user.auth_basic();
                http_request.headers_mut().typed_insert(BasicAuthorization::basic(
                    auth.username(),
                    &if ok {
                        auth.password().to_owned()
                    } else {
                        Password(16..32).fake::<String>()
                    },
                ));
            } else {
                let auth = user.auth_bearer().await;
                http_request.headers_mut().typed_insert(if ok {
                    auth
                } else {
                    BearerAuthorization::bearer(&Faker.fake::<Uuid>().to_string()).unwrap()
                });
            }
        }

        if auth {
            let request = Authenticated::<Request>::from_request(http_request, mock.state()).await;

            if ok {
                let request = request.unwrap();
                assert_eq!(request.user.id, user.id());
                assert_eq!(request.validated.request, body);
                assert_eq!(request.validated.ty, Type::Json);
            } else {
                let error = request.unwrap_err();
                assert_eq!(error.ty, Type::Json);
            }
        } else {
            let request =
                Validated::<Request>::from_request(http_request, mock.state()).await.unwrap();
            assert_eq!(request.request, body);
            assert_eq!(request.ty, Type::Json);
        }
    }

    mod unit {
        use super::*;

        #[api_derive(fake = true)]
        #[endpoint(path = "test", url_only = true, api_crate = nghe_api)]
        #[derive(Clone, Copy, PartialEq)]
        struct Request;

        #[rstest]
        #[tokio::test]
        async fn test_from_request(
            #[future(awt)] mock: Mock,
            #[values(true, false)] auth: bool,
            #[values(true, false)] ok: bool,
            #[values(true, false)] use_password: bool,
        ) {
            let user = mock.user(0).await;

            let mut http_request =
                http::Request::builder().method(http::Method::GET).body(Body::empty()).unwrap();
            if auth {
                if use_password {
                    let auth = user.auth_basic();
                    http_request.headers_mut().typed_insert(BasicAuthorization::basic(
                        auth.username(),
                        &if ok {
                            auth.password().to_owned()
                        } else {
                            Password(16..32).fake::<String>()
                        },
                    ));
                } else {
                    let auth = user.auth_bearer().await;
                    http_request.headers_mut().typed_insert(if ok {
                        auth
                    } else {
                        BearerAuthorization::bearer(&Faker.fake::<Uuid>().to_string()).unwrap()
                    });
                }
            }

            if auth {
                let request =
                    Authenticated::<Request>::from_request(http_request, mock.state()).await;

                if ok {
                    let request = request.unwrap();
                    assert_eq!(request.user.id, user.id());
                    assert_eq!(request.validated.request, Request);
                    assert_eq!(request.validated.ty, Type::Json);
                } else {
                    let error = request.unwrap_err();
                    assert_eq!(error.ty, Type::Json);
                }
            } else {
                let request =
                    Validated::<Request>::from_request(http_request, mock.state()).await.unwrap();
                assert_eq!(request.request, Request);
                assert_eq!(request.ty, Type::Json);
            }
        }
    }
}
