use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest, Request};
use axum::RequestExt;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::auth::{Auth, BinaryRequest};
use nghe_api::common::Endpoint;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::users;
use crate::Error;

#[derive(Debug)]
pub struct GetUser<R> {
    pub id: Uuid,
    pub request: R,
}

#[derive(Debug)]
pub struct PostUser<R> {
    pub id: Uuid,
    pub request: R,
}

#[derive(Debug)]
pub struct BinaryUser<R, const NEED_AUTH: bool> {
    pub id: Uuid,
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

async fn json_authenticate<S>(
    query: &str,
    state: &S,
) -> Result<(Database, Uuid, users::Role), Error>
where
    Database: FromRef<S>,
{
    let auth: Auth = serde_html_form::from_str(query)
        .map_err(|_| Error::SerializeRequest("invalid auth parameters"))?;

    let database = Database::from_ref(state);
    let (id, role) = authenticate(&database, auth).await?;

    Ok((database, id, role))
}

#[async_trait::async_trait]
impl<S, R> FromRequest<S> for GetUser<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
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
        Ok(Self { id, request: request.authorize(role)? })
    }
}

#[async_trait::async_trait]
impl<S, R> FromRequest<S> for PostUser<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
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
        Ok(Self { id, request: request.authorize(role)? })
    }
}

#[async_trait::async_trait]
impl<S, R, const NEED_AUTH: bool> FromRequest<S> for BinaryUser<R, NEED_AUTH>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: Endpoint + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes: Bytes = request.extract().await?;

        if NEED_AUTH {
            let BinaryRequest::<R> { auth, request } = bitcode::decode(&bytes)
                .map_err(|_| Error::SerializeRequest("invalid request body"))?;

            let database = Database::from_ref(state);
            let (id, role) = authenticate(&database, auth).await?;
            Ok(Self { id, request: request.authorize(role)? })
        } else {
            Ok(Self {
                id: Uuid::default(),
                request: bitcode::decode(&bytes)
                    .map_err(|_| Error::SerializeRequest("invalid request body"))?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http;
    use concat_string::concat_string;
    use fake::{Dummy, Fake, Faker};
    use nghe_proc_macro::api_derive;
    use rstest::rstest;
    use serde::Serialize;

    use super::*;
    use crate::test::{mock, Mock};

    #[api_derive(endpoint = true)]
    #[endpoint(path = "test", same_crate = false)]
    #[derive(Clone, Copy, Serialize, Dummy, PartialEq, Eq)]
    struct Request {
        param_one: i32,
        param_two: u32,
    }

    #[api_derive]
    #[allow(dead_code)]
    struct Response;

    impl Authorize for Request {
        fn authorize(self, _: users::Role) -> Result<Self, Error> {
            Ok(self)
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let id =
            authenticate(mock.database(), (&mock.user(0).await.auth()).into()).await.unwrap().0;
        assert_eq!(id, mock.user(0).await.user.id);
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_wrong_username(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let auth = mock.user(0).await.auth();

        let username: String = Faker.fake();
        let auth = Auth { username: &username, ..(&auth).into() };
        assert!(authenticate(mock.database(), auth).await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_wrong_password(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let auth = mock.user(0).await.auth();

        let token = Auth::tokenize(Faker.fake::<String>(), &auth.salt);
        let auth = Auth { token, ..(&auth).into() };
        assert!(authenticate(mock.database(), auth).await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_json_get(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        #[derive(Debug, Serialize)]
        struct RequestAuth<'u, 't> {
            #[serde(flatten, borrow)]
            auth: Auth<'u, 't>,
            #[serde(flatten)]
            request: Request,
        }

        let request: Request = Faker.fake();

        let user = mock.user(0).await;
        let auth = user.auth();
        let auth = (&auth).into();

        let http_request = http::Request::builder()
            .method(http::Method::GET)
            .uri(concat_string!(
                Request::ENDPOINT,
                "?",
                serde_html_form::to_string(RequestAuth { auth, request }).unwrap()
            ))
            .body(Body::empty())
            .unwrap();

        let test_request =
            GetUser::<Request>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(user.user.id, test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_json_post(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();

        let user = mock.user(0).await;
        let auth = user.auth();
        let auth = (&auth).into();

        let http_request = http::Request::builder()
            .method(http::Method::POST)
            .uri(concat_string!(
                Request::ENDPOINT,
                "?",
                serde_html_form::to_string::<Auth>(auth).unwrap()
            ))
            .body(Body::from(serde_html_form::to_string(request).unwrap()))
            .unwrap();

        let test_request =
            PostUser::<Request>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(user.user.id, test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_binary_auth(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();

        let user = mock.user(0).await;
        let auth = user.auth();
        let auth = (&auth).into();

        let http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(Body::from(bitcode::encode(&BinaryRequest { auth, request })))
            .unwrap();

        let test_request =
            BinaryUser::<Request, true>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(user.user.id, test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_binary_no_auth(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();

        let http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(Body::from(bitcode::encode(&request)))
            .unwrap();

        let test_request =
            BinaryUser::<Request, false>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(Uuid::default(), test_request.id);
        assert_eq!(request, test_request.request);
    }
}
