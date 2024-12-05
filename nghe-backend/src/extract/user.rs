use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::headers::{self, HeaderMapExt};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::users;
use crate::Error;

pub struct Token {
    id: Uuid,
}

pub struct Header {
    id: Uuid,
}

impl<S> FromRequestParts<S> for Header
where
    S: Send + Sync,
    Database: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .typed_get::<headers::Authorization<headers::authorization::Basic>>()
            .ok_or_else(|| Error::MissingAuthenticationHeader)?;

        let database = Database::from_ref(state);
        let users::Auth { id, password, role } = users::table
            .filter(users::username.eq(header.username()))
            .select(users::Auth::as_select())
            .first(&mut database.get().await?)
            .await
            .map_err(|_| Error::Unauthenticated)?;

        let password = database.decrypt(password)?;
        if header.password().as_bytes() == password {
            Ok(Self { id })
        } else {
            Err(Error::Unauthenticated)
        }
    }
}
