use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest, Request};
use axum::RequestExt;
use axum_extra::headers::{self, HeaderMapExt};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::auth::Auth;
use nghe_api::common::{BinaryRequest, FormRequest, JsonRequest};
use uuid::Uuid;

use crate::database::Database;
use crate::orm::users;
use crate::Error;

#[derive(Debug)]
pub struct FormGetUser<R, const NEED_AUTH: bool> {
    pub id: Uuid,
    pub request: R,
}

#[derive(Debug)]
pub struct FormPostUser<R> {
    pub id: Uuid,
    pub request: R,
}

#[derive(Debug)]
pub struct BinaryUser<R, const NEED_AUTH: bool> {
    #[allow(dead_code)]
    pub id: Uuid,
    pub request: R,
}

#[derive(Debug)]
pub struct JsonUser<R, const NEED_AUTH: bool> {
    #[allow(dead_code)]
    pub id: Uuid,
    pub request: R,
}

trait FromIdRequest<R>: Sized {
    fn from_id_request(id: Uuid, request: R) -> Self;
}

pub trait Authorize {
    fn authorize(role: users::Role) -> Result<(), Error>;
}

async fn authenticate_token<A: Authorize>(
    database: &Database,
    auth: Auth<'_, '_>,
) -> Result<Uuid, Error> {
    let users::Auth { id, password, role } = users::table
        .filter(users::username.eq(&auth.username))
        .select(users::Auth::as_select())
        .first(&mut database.get().await?)
        .await
        .map_err(|_| Error::Unauthenticated)?;
    A::authorize(role)?;
    let password = database.decrypt(password)?;
    if auth.check(password) { Ok(id) } else { Err(Error::Unauthenticated) }
}

async fn authenticate_header<A: Authorize>(
    database: &Database,
    auth: headers::Authorization<headers::authorization::Basic>,
) -> Result<Uuid, Error> {
    let users::Auth { id, password, role } = users::table
        .filter(users::username.eq(auth.username()))
        .select(users::Auth::as_select())
        .first(&mut database.get().await?)
        .await
        .map_err(|_| Error::Unauthenticated)?;
    A::authorize(role)?;
    let password = database.decrypt(password)?;
    if auth.password().as_bytes() == password { Ok(id) } else { Err(Error::Unauthenticated) }
}

// TODO: Optimize this after https://github.com/serde-rs/serde/issues/1183
async fn form_authenticate<S, R: FormRequest + Authorize, U: FromIdRequest<R>>(
    state: &S,
    input: &str,
) -> Result<U, Error>
where
    Database: FromRef<S>,
{
    let auth: Auth = serde_html_form::from_str(input)
        .map_err(|_| Error::SerializeAuthParameters(input.to_owned()))?;

    let database = Database::from_ref(state);
    let id = authenticate_token::<R>(&database, auth).await?;

    let request = serde_html_form::from_str::<R>(input)
        .map_err(|_| Error::SerializeRequestParameters(input.to_owned()))?;

    Ok(U::from_id_request(id, request))
}

impl<R, const NEED_AUTH: bool> FromIdRequest<R> for FormGetUser<R, NEED_AUTH> {
    fn from_id_request(id: Uuid, request: R) -> Self {
        Self { id, request }
    }
}

impl<R> FromIdRequest<R> for FormPostUser<R> {
    fn from_id_request(id: Uuid, request: R) -> Self {
        Self { id, request }
    }
}

impl<S, R, const NEED_AUTH: bool> FromRequest<S> for FormGetUser<R, NEED_AUTH>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: FormRequest + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let query = request.uri().query();

        if NEED_AUTH {
            form_authenticate(state, query.ok_or_else(|| Error::GetRequestMissingQueryParameters)?)
                .await
        } else {
            let query = query.unwrap_or_default();
            Ok(Self {
                id: Uuid::default(),
                request: serde_html_form::from_str(query)
                    .map_err(|_| Error::SerializeRequestParameters(query.to_owned()))?,
            })
        }
    }
}

impl<S, R> FromRequest<S> for FormPostUser<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: FormRequest + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        form_authenticate(state, &request.extract::<String, _>().await?).await
    }
}

impl<S, R, const NEED_AUTH: bool> FromRequest<S> for BinaryUser<R, NEED_AUTH>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: BinaryRequest + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let id = if NEED_AUTH {
            let database = Database::from_ref(state);
            let auth = request
                .headers()
                .typed_get::<headers::Authorization<headers::authorization::Basic>>()
                .ok_or_else(|| Error::MissingAuthenticationHeader)?;
            authenticate_header::<R>(&database, auth).await?
        } else {
            Uuid::default()
        };

        Ok(Self {
            id,
            request: bitcode::deserialize(&request.extract::<Bytes, _>().await?)
                .map_err(|_| Error::SerializeBinaryRequest)?,
        })
    }
}

impl<S, R, const NEED_AUTH: bool> FromRequest<S> for JsonUser<R, NEED_AUTH>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: JsonRequest + Authorize + Send,
{
    type Rejection = Error;

    #[tracing::instrument(skip_all, err)]
    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let id = if NEED_AUTH {
            let database = Database::from_ref(state);
            let auth = request
                .headers()
                .typed_get::<headers::Authorization<headers::authorization::Basic>>()
                .ok_or_else(|| Error::MissingAuthenticationHeader)?;
            authenticate_header::<R>(&database, auth).await?
        } else {
            Uuid::default()
        };

        Ok(Self {
            id,
            request: axum::Json::from_request(request, &())
                .await
                .map(|value| value.0)
                .map_err(|e| Error::SerializeJsonRequest(e.to_string()))?,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(unexpected_cfgs)]

    use axum::body::Body;
    use axum::http;
    use concat_string::concat_string;
    use fake::{Fake, Faker};
    use headers::ContentType;
    use nghe_api::auth::Token;
    use nghe_api::common::FormRequest as _;
    use nghe_proc_macro::api_derive;
    use rstest::rstest;
    use serde::Serialize;

    use super::*;
    use crate::test::{mock, Mock};

    #[api_derive(fake = true)]
    #[endpoint(path = "test", binary = true, json = true, same_crate = false)]
    #[derive(Clone, Copy)]
    struct Request {
        param_one: i32,
        param_two: u32,
    }

    #[api_derive]
    #[allow(dead_code)]
    struct Response;

    impl Authorize for Request {
        fn authorize(_: users::Role) -> Result<(), Error> {
            Ok(())
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_token(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let user = mock.user(0).await;
        let id = authenticate_token::<Request>(mock.database(), user.auth_token()).await.unwrap();
        assert_eq!(id, user.id());
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_token_wrong_username(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let auth = mock.user(0).await.auth_token();
        let auth = Auth { username: Faker.fake::<String>().into(), ..auth };
        assert!(authenticate_token::<Request>(mock.database(), auth).await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_token_wrong_password(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let auth = mock.user(0).await.auth_token();
        let token = Token::new(Faker.fake::<String>(), auth.salt.as_bytes());
        let auth = Auth { token, ..auth };
        assert!(authenticate_token::<Request>(mock.database(), auth).await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_header(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let user = mock.user(0).await;
        let id = authenticate_header::<Request>(mock.database(), user.auth_header()).await.unwrap();
        assert_eq!(id, user.id());
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_header_wrong_username(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let auth = mock.user(0).await.auth_header();
        let auth = headers::Authorization::basic(&Faker.fake::<String>(), auth.password());
        assert!(authenticate_header::<Request>(mock.database(), auth).await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_header_wrong_password(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let auth = mock.user(0).await.auth_header();
        let auth = headers::Authorization::basic(auth.username(), &Faker.fake::<String>());
        assert!(authenticate_header::<Request>(mock.database(), auth).await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_form_get_auth(
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

        let http_request = http::Request::builder()
            .method(http::Method::GET)
            .uri(concat_string!(
                Request::URL_FORM,
                "?",
                serde_html_form::to_string(RequestAuth { auth: user.auth_token(), request })
                    .unwrap()
            ))
            .body(Body::empty())
            .unwrap();

        let test_request =
            FormGetUser::<Request, true>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(user.user.id, test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_form_get_no_auth(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();

        let http_request = http::Request::builder()
            .method(http::Method::GET)
            .uri(concat_string!(
                Request::URL_FORM,
                "?",
                serde_html_form::to_string(request).unwrap()
            ))
            .body(Body::empty())
            .unwrap();

        let test_request =
            FormGetUser::<Request, false>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(Uuid::default(), test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_form_post(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();
        let user = mock.user(0).await;

        let http_request = http::Request::builder()
            .method(http::Method::POST)
            .uri(Request::URL_FORM)
            .body(Body::from(concat_string!(
                serde_html_form::to_string(user.auth_token()).unwrap(),
                "&",
                serde_html_form::to_string(request).unwrap()
            )))
            .unwrap();

        let test_request =
            FormPostUser::<Request>::from_request(http_request, mock.state()).await.unwrap();
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

        let mut http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(Body::from(bitcode::serialize(&request).unwrap()))
            .unwrap();
        http_request.headers_mut().typed_insert(user.auth_header());

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
            .body(Body::from(bitcode::serialize(&request).unwrap()))
            .unwrap();

        let test_request =
            BinaryUser::<Request, false>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(Uuid::default(), test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_json_auth(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();
        let user = mock.user(0).await;

        let mut http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(Body::from(serde_json::to_string(&request).unwrap()))
            .unwrap();
        http_request.headers_mut().typed_insert(user.auth_header());
        http_request.headers_mut().typed_insert(ContentType::json());

        let test_request =
            JsonUser::<Request, true>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(user.user.id, test_request.id);
        assert_eq!(request, test_request.request);
    }

    #[rstest]
    #[tokio::test]
    async fn test_json_no_auth(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        let request: Request = Faker.fake();

        let mut http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(Body::from(serde_json::to_string(&request).unwrap()))
            .unwrap();
        http_request.headers_mut().typed_insert(ContentType::json());

        let test_request =
            JsonUser::<Request, false>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(Uuid::default(), test_request.id);
        assert_eq!(request, test_request.request);
    }
}
