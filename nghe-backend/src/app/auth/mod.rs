use axum::extract::{FromRef, FromRequest, Request};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::common::{Auth, Endpoint};
use uuid::Uuid;

use super::error::Error;
use super::state::{App, Database};
use crate::orm::users;

#[derive(Debug)]
pub struct GetUser<R> {
    pub id: Uuid,
    pub role: users::Role,
    pub request: R,
}

pub trait Authorize: Sized {
    fn authorize(self, role: users::Role) -> Result<Self, Error>;
}

async fn authenticate(
    database: &Database,
    data: Auth<'_, '_>,
) -> Result<(Uuid, users::Role), Error> {
    let (id, encrypted_password, role) = users::dsl::table
        .filter(users::dsl::username.eq(data.username))
        .select((users::dsl::id, users::dsl::password, users::Role::as_select()))
        .first::<(Uuid, Vec<u8>, users::Role)>(&mut database.get().await?)
        .await
        .map_err(|_| Error::Unauthenticated)?;
    let password = database.decrypt(encrypted_password)?;

    if Auth::check(password, data.salt, &data.token) {
        Ok((id, role))
    } else {
        Err(Error::Unauthenticated)
    }
}

#[async_trait::async_trait]
impl<S, R> FromRequest<S> for GetUser<R>
where
    S: Send + Sync,
    App: FromRef<S>,
    R: Endpoint + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let query =
            request.uri().query().ok_or_else(|| Error::BadRequest("missing query parameters"))?;

        // TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
        let auth: Auth = serde_html_form::from_str(query)
            .map_err(|_| Error::BadRequest("invalid auth parameters"))?;
        let app = App::from_ref(state);
        let (id, role) = authenticate(&app.database, auth).await?;

        let request: R = serde_html_form::from_str(query)
            .map_err(|_| Error::BadRequest("invalid request parameters"))?;

        Ok(Self { id, role, request: request.authorize(role)? })
    }
}
