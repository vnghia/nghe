use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest, Request};
use axum::RequestExt;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_api::auth::{Auth, AuthRequest};
use nghe_api::common::{BinaryRequest, FormRequest};
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
    pub id: Uuid,
    pub request: R,
}

trait FromIdRequest<R>: Sized {
    fn from_id_request(id: Uuid, request: R) -> Self;
}

pub trait Authorize {
    fn authorize(role: users::Role) -> Result<(), Error>;
}

async fn authenticate<A: Authorize>(
    database: &Database,
    data: Auth<'_, '_>,
) -> Result<Uuid, Error> {
    let users::Auth { id, password, role } = users::table
        .filter(users::username.eq(data.username))
        .select(users::Auth::as_select())
        .first(&mut database.get().await?)
        .await
        .map_err(|_| Error::Unauthenticated)?;
    let password = database.decrypt(password)?;
    A::authorize(role)?;

    if Auth::check(password, data.salt.as_bytes(), &data.token) {
        Ok(id)
    } else {
        Err(Error::Unauthenticated)
    }
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
    let id = authenticate::<R>(&database, auth).await?;

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
        let bytes: Bytes = request.extract().await?;

        if NEED_AUTH {
            let AuthRequest::<R> { auth, request } =
                bitcode::deserialize(&bytes).map_err(|_| Error::SerializeBinaryRequest)?;

            let database = Database::from_ref(state);
            let id = authenticate::<R>(&database, auth).await?;
            Ok(Self { id, request })
        } else {
            Ok(Self {
                id: Uuid::default(),
                request: bitcode::deserialize(&bytes).map_err(|_| Error::SerializeBinaryRequest)?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unexpected_cfgs)]

    use axum::body::Body;
    use axum::http;
    use concat_string::concat_string;
    use fake::{Fake, Faker};
    use nghe_api::common::FormRequest as _;
    use nghe_proc_macro::api_derive;
    use rstest::rstest;
    use serde::Serialize;

    use super::*;
    use crate::test::{mock, Mock};

    #[api_derive(fake = true)]
    #[endpoint(path = "test", same_crate = false)]
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
    async fn test_authenticate(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let user = mock.user(0).await;
        let id = authenticate::<Request>(mock.database(), user.auth()).await.unwrap();
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
        let auth = Auth { username: Faker.fake::<String>().into(), ..auth };
        assert!(authenticate::<Request>(mock.database(), auth).await.is_err());
    }

    #[rstest]
    #[tokio::test]
    async fn test_authenticate_wrong_password(
        #[future(awt)]
        #[with(1, 0)]
        mock: Mock,
    ) {
        let auth = mock.user(0).await.auth();
        let token = Auth::tokenize(Faker.fake::<String>(), auth.salt.as_bytes());
        let auth = Auth { token, ..auth };
        assert!(authenticate::<Request>(mock.database(), auth).await.is_err());
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
                serde_html_form::to_string(RequestAuth { auth: user.auth(), request }).unwrap()
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
                serde_html_form::to_string(user.auth()).unwrap(),
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

        let http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(Body::from(
                bitcode::serialize(&AuthRequest { auth: user.auth(), request }).unwrap(),
            ))
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
            .body(Body::from(bitcode::serialize(&request).unwrap()))
            .unwrap();

        let test_request =
            BinaryUser::<Request, false>::from_request(http_request, mock.state()).await.unwrap();
        assert_eq!(Uuid::default(), test_request.id);
        assert_eq!(request, test_request.request);
    }
}
