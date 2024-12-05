use std::marker::PhantomData;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::headers::{self, HeaderMapExt};
use uuid::Uuid;

use super::{login, AuthN, AuthZ};
use crate::database::Database;
use crate::Error;

pub struct Header<R> {
    id: Uuid,
    _request: PhantomData<R>,
}

type Type = headers::Authorization<headers::authorization::Basic>;

impl AuthN for Type {
    fn username(&self) -> &str {
        self.username()
    }

    fn is_authenticated(&self, password: impl AsRef<[u8]>) -> bool {
        self.password().as_bytes() == password
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
            .typed_get::<headers::Authorization<headers::authorization::Basic>>()
            .ok_or_else(|| Error::MissingAuthenticationHeader)?;
        let id = login::<R, _>(state, &header).await?;
        Ok(Self { id, _request: PhantomData::default() })
    }
}
