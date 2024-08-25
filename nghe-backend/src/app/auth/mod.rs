use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest, Request};
use axum::RequestExt;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::auth::{Auth, BinaryRequest};
use nghe_api::common::Endpoint;
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

#[derive(Debug)]
pub struct PostUser<R> {
    pub id: Uuid,
    pub role: users::Role,
    pub request: R,
}

#[derive(Debug)]
pub struct BinaryUser<R, const NEED_AUTH: bool> {
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
    let users::Auth { id, password, role } = users::table
        .filter(users::schema::username.eq(data.username))
        .select(users::Auth::as_select())
        .first(&mut database.get().await?)
        .await
        .map_err(|_| Error::Unauthenticated)?;
    let password = database.decrypt(password)?;

    if Auth::check(password, data.salt, &data.token) {
        Ok((id, role))
    } else {
        Err(Error::Unauthenticated)
    }
}

async fn json_authenticate<S>(query: &str, state: &S) -> Result<(App, Uuid, users::Role), Error>
where
    App: FromRef<S>,
{
    let auth: Auth = serde_html_form::from_str(query)
        .map_err(|_| Error::SerializeRequest("invalid auth parameters"))?;

    let app = App::from_ref(state);
    let (id, role) = authenticate(&app.database, auth).await?;

    Ok((app, id, role))
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
        let query = request
            .uri()
            .query()
            .ok_or_else(|| Error::SerializeRequest("missing query parameters"))?;

        // TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
        let (_, id, role) = json_authenticate(query, state).await?;
        let request: R = serde_html_form::from_str(query)
            .map_err(|_| Error::SerializeRequest("invalid request parameters"))?;
        Ok(Self { id, role, request: request.authorize(role)? })
    }
}

#[async_trait::async_trait]
impl<S, R> FromRequest<S> for PostUser<R>
where
    S: Send + Sync,
    App: FromRef<S>,
    R: Endpoint + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let query = request
            .uri()
            .query()
            .ok_or_else(|| Error::SerializeRequest("missing query parameters"))?;

        let (_, id, role) = json_authenticate(query, state).await?;
        let request: R = serde_html_form::from_bytes(&request.extract::<Bytes, _>().await?)
            .map_err(|_| Error::SerializeRequest("invalid request body"))?;
        Ok(Self { id, role, request: request.authorize(role)? })
    }
}

#[async_trait::async_trait]
impl<S, R, const NEED_AUTH: bool> FromRequest<S> for BinaryUser<R, NEED_AUTH>
where
    S: Send + Sync,
    App: FromRef<S>,
    R: Endpoint + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes: Bytes = request.extract().await?;

        if NEED_AUTH {
            let BinaryRequest::<R> { auth, request } = bitcode::decode(&bytes)
                .map_err(|_| Error::SerializeRequest("invalid request body"))?;

            let app = App::from_ref(state);
            let (id, role) = authenticate(&app.database, auth).await?;
            Ok(Self { id, role, request: request.authorize(role)? })
        } else {
            Ok(Self {
                id: Uuid::default(),
                role: users::Role::default(),
                request: bitcode::decode(&bytes)
                    .map_err(|_| Error::SerializeRequest("invalid request body"))?,
            })
        }
    }
}
