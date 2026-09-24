use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest};
use axum::http::Method;
use axum_extra::headers::{self, HeaderMapExt};

use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

#[derive(Debug)]
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
    R: for<'form> nghe_api::common::Request<'form, 'form, 'form, 'form, 'form> + Send,
{
    type Rejection = Error;

    async fn from_request(
        request: axum::extract::Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let method = request.method().to_owned();

        if method == Method::GET {
            return Self::from_form(request.uri().query().unwrap_or_default().as_bytes());
        }

        if method == Method::POST {
            let headers = request.headers();
            let content_type = headers
                .typed_get::<headers::ContentType>()
                .ok_or_else(|| error::Kind::MissingContentTypeHeader)?;

            if content_type == headers::ContentType::form_url_encoded() {
                let body = Bytes::from_request(request, state).await.map_err(error::Kind::from)?;
                return Self::from_form(&body);
            }

            let body = Bytes::from_request(request, state).await.map_err(error::Kind::from)?;
            if content_type == headers::ContentType::json() {
                return Ok(Self {
                    ty: Type::Json,
                    request: serde_json::from_slice(&body).map_err(error::Kind::from)?,
                });
            }
        }

        error::Kind::MethodNotAllowed(method).into()
    }
}

impl<S, R> FromRequest<S> for Authenticated<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: for<'form> nghe_api::common::Request<'form, 'form, 'form, 'form, 'form> + Send,
{
    type Rejection = Error;

    async fn from_request(
        request: axum::extract::Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let database = Database::from_ref(state);
        let method = request.method().to_owned();

        if method == Method::GET {
            return Self::from_form(
                &database,
                &request.uri().query().unwrap_or_default().as_bytes(),
            )
            .await;
        }

        if method == Method::POST {
            let headers = request.headers();
            let content_type = headers
                .typed_get::<headers::ContentType>()
                .ok_or_else(|| error::Kind::MissingContentTypeHeader)?;

            if content_type == headers::ContentType::form_url_encoded() {
                let body = Bytes::from_request(request, state).await.map_err(error::Kind::from)?;
                return Self::from_form(&database, &body).await;
            }

            let user = users::Authenticated::from_headers(&database, headers).await?;
            let body = Bytes::from_request(request, state).await.map_err(error::Kind::from)?;
            if content_type == headers::ContentType::json() {
                return Ok(Self {
                    validated: Validated {
                        ty: Type::Json,
                        request: serde_json::from_slice(&body).map_err(error::Kind::from)?,
                    },
                    user,
                });
            }
        }

        error::Kind::MethodNotAllowed(method).into()
    }
}
