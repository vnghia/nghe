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
pub struct GetUser<R, const NEED_AUTH: bool> {
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

trait FromIdRequest<R>: Sized {
    fn from_id_request(id: Uuid, request: R) -> Self;
}

pub trait Authorize: Sized {
    fn authorize(self, role: users::Role) -> Result<Self, Error>;
}

async fn authenticate(
    database: &Database,
    data: Auth<'_, '_>,
) -> Result<(Uuid, users::Role), Error> {
    let users::Auth { id, password, role } = users::table
        .filter(users::username.eq(data.username))
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

// TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
async fn json_authenticate<S, R: Endpoint + Authorize, U: FromIdRequest<R>>(
    state: &S,
    input: &str,
) -> Result<U, Error>
where
    Database: FromRef<S>,
{
    let auth: Auth = serde_html_form::from_str(input)
        .map_err(|_| Error::SerializeAuthParameters(input.to_owned()))?;

    let database = Database::from_ref(state);
    let (id, role) = authenticate(&database, auth).await?;

    let request = serde_html_form::from_str::<R>(input)
        .map_err(|_| Error::SerializeRequestParameters(input.to_owned()))?
        .authorize(role)?;

    Ok(U::from_id_request(id, request))
}

impl<R, const NEED_AUTH: bool> FromIdRequest<R> for GetUser<R, NEED_AUTH> {
    fn from_id_request(id: Uuid, request: R) -> Self {
        Self { id, request }
    }
}

impl<R> FromIdRequest<R> for PostUser<R> {
    fn from_id_request(id: Uuid, request: R) -> Self {
        Self { id, request }
    }
}

#[async_trait::async_trait]
impl<S, R, const NEED_AUTH: bool> FromRequest<S> for GetUser<R, NEED_AUTH>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: Endpoint + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let query = request.uri().query().ok_or_else(|| Error::GetRequestMissingQueryParameters)?;

        if NEED_AUTH {
            json_authenticate(state, query).await
        } else {
            Ok(Self {
                id: Uuid::default(),
                request: serde_html_form::from_str(query)
                    .map_err(|_| Error::SerializeRequestParameters(query.to_owned()))?,
            })
        }
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
        json_authenticate(state, &request.extract::<String, _>().await?).await
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
            let BinaryRequest::<R> { auth, request } =
                bitcode::decode(&bytes).map_err(|_| Error::SerializeBinaryRequest)?;

            let database = Database::from_ref(state);
            let (id, role) = authenticate(&database, auth).await?;
            Ok(Self { id, request: request.authorize(role)? })
        } else {
            Ok(Self {
                id: Uuid::default(),
                request: bitcode::decode(&bytes).map_err(|_| Error::SerializeBinaryRequest)?,
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
        let user = mock.user(0).await;
        let id = authenticate(mock.database(), (&user.auth()).into()).await.unwrap().0;
        assert_eq!(id, user.id());
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
    async fn test_json_get_auth(
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
            GetUser::<Request, true>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(user.user.id, test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_json_get_no_auth(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();

        let http_request = http::Request::builder()
            .method(http::Method::GET)
            .uri(concat_string!(
                Request::ENDPOINT,
                "?",
                serde_html_form::to_string(request).unwrap()
            ))
            .body(Body::empty())
            .unwrap();

        let test_request =
            GetUser::<Request, false>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(Uuid::default(), test_request.id);
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
            .uri(Request::ENDPOINT)
            .body(Body::from(concat_string!(
                serde_html_form::to_string::<Auth>(auth).unwrap(),
                "&",
                serde_html_form::to_string(request).unwrap()
            )))
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
