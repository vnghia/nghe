use std::marker::PhantomData;

use anyhow::Result;
use axum::extract::{FromRef, FromRequest, Request};
use axum::http::Method;
use axum_extra::extract::Form;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use nghe_types::params::CommonParams;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use super::error::ServerError;
use crate::models::*;
use crate::utils::password::*;
use crate::{Database, OSError};

async fn validate<const REQUIRED_ROLE: users::Role>(
    Database { pool, key }: &Database,
    common_params: impl AsRef<CommonParams>,
) -> Result<(Uuid, users::Role)> {
    let common_params = common_params.as_ref();
    let (user_id, user_password, user_role) = match users::table
        .filter(users::username.eq(&common_params.username))
        .select((users::id, users::password, users::Role::as_select()))
        .first::<(Uuid, Vec<u8>, users::Role)>(&mut pool.get().await?)
        .await
    {
        Ok(res) => res,
        _ => anyhow::bail!(OSError::Unauthorized),
    };

    check_password(
        decrypt_password(key, user_password)?,
        &common_params.salt,
        &common_params.token,
    )?;
    if REQUIRED_ROLE > user_role {
        anyhow::bail!(OSError::Forbidden("access admin endpoint".into()));
    }
    Ok((user_id, user_role))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedForm<R, P, const REQUIRED_ROLE: users::Role> {
    pub params: P,
    pub user_id: Uuid,
    pub user_role: users::Role,
    pub phantom: PhantomData<R>,
}

#[async_trait::async_trait]
impl<R, P, const REQUIRED_ROLE: users::Role, S> FromRequest<S>
    for ValidatedForm<R, P, REQUIRED_ROLE>
where
    P: DeserializeOwned + Send,
    R: DeserializeOwned + Send + Sync + AsRef<CommonParams> + Into<P>,
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (user_id, user_role, params) = match *req.method() {
            Method::GET => {
                let Form(params) = Form::<R>::from_request(req, state)
                    .await
                    .map_err(std::convert::Into::<OSError>::into)?;
                let database = Database::from_ref(state);
                let (user_id, user_role) = validate::<REQUIRED_ROLE>(&database, &params).await?;
                (user_id, user_role, params.into())
            }
            Method::POST => {
                let common: CommonParams =
                    serde_html_form::from_str(req.uri().query().ok_or_else(|| {
                        OSError::InvalidParameter("authentication params are missing".into())
                    })?)
                    .map_err(|_| {
                        OSError::InvalidParameter("authentication params are incorrect".into())
                    })?;
                let Form(params) = Form::<P>::from_request(req, state)
                    .await
                    .map_err(std::convert::Into::<OSError>::into)?;
                let database = Database::from_ref(state);
                let (user_id, user_role) = validate::<REQUIRED_ROLE>(&database, &common).await?;
                (user_id, user_role, params)
            }
            _ => unreachable!("method is not allowed"),
        };

        Ok(ValidatedForm { params, user_id, user_role, phantom: PhantomData })
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::uri;
    use concat_string::concat_string;
    use fake::faker::internet::en::*;
    use fake::{Fake, Faker};
    use nghe_proc_macros::{add_common_convert, add_common_validate};
    use nghe_types::params::{to_password_token, WithCommon};

    use super::*;
    use crate::utils::test::Infra;

    #[add_common_convert]
    struct TestParams {}

    #[add_common_convert]
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestWithArgsParams {
        arg1: i32,
        arg2: String,
    }
    add_common_validate!(TestWithArgsParams);

    #[tokio::test]
    async fn test_validate_success() {
        let infra = Infra::new().await.add_user(None).await;
        assert!(
            validate::<{ users::Role::const_default() }>(
                infra.database(),
                TestParams {}.with_common(infra.to_common_params(0))
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validate_wrong_username() {
        let infra = Infra::new().await.add_user(None).await;
        let wrong_username: String = Username().fake();
        assert!(matches!(
            validate::<{ users::Role::const_default() }>(
                infra.database(),
                TestParams {}.with_common(CommonParams {
                    username: wrong_username,
                    ..infra.to_common_params(0)
                })
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::Unauthorized
        ));
    }

    #[tokio::test]
    async fn test_validate_wrong_password() {
        let infra = Infra::new().await.add_user(None).await;

        let username = infra.users[0].basic.username.to_string();
        let client_salt = Password(8..16).fake::<String>();
        let client_token = to_password_token(Password(16..32).fake::<String>(), &client_salt);

        assert!(matches!(
            validate::<{ users::Role::const_default() }>(
                infra.database(),
                TestParams {}.with_common(CommonParams {
                    username,
                    salt: client_salt,
                    token: client_token
                })
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::Unauthorized
        ));
    }

    #[tokio::test]
    async fn test_validate_admin_success() {
        let infra = Infra::new()
            .await
            .add_user(Some(users::Role { admin_role: true, ..users::Role::const_default() }))
            .await;
        assert!(
            validate::<{ users::Role { admin_role: true, ..users::Role::const_default() } }>(
                infra.database(),
                TestParams {}.with_common(infra.to_common_params(0))
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validate_no_admin() {
        let infra = Infra::new().await.add_user(None).await;
        assert!(matches!(
            validate::<{ users::Role { admin_role: true, ..users::Role::const_default() } }>(
                infra.database(),
                TestParams {}.with_common(infra.to_common_params(0))
            )
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::Forbidden(_)
        ));
    }

    #[tokio::test]
    async fn test_from_request_get() {
        let infra = Infra::new().await.add_user(None).await;
        let params = TestWithArgsParams { ..Faker.fake() };

        let req = Request::builder()
            .method("GET")
            .uri(
                uri::Builder::new()
                    .path_and_query(concat_string!(
                        "/rest?",
                        serde_html_form::to_string(
                            params.clone().with_common(infra.to_common_params(0)),
                        )
                        .unwrap()
                    ))
                    .build()
                    .unwrap(),
            )
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            TestWithArgsRequest::from_request(req, &infra.state()).await.unwrap().params,
            params
        )
    }

    #[tokio::test]
    async fn test_from_request_post() {
        let infra = Infra::new().await.add_user(None).await;
        let params = TestWithArgsParams { ..Faker.fake() };

        let req: axum::http::Request<Body> = Request::builder()
            .method("POST")
            .header("content-type", "application/x-www-form-urlencoded")
            .uri(
                uri::Builder::new()
                    .path_and_query(concat_string!(
                        "/rest?",
                        serde_html_form::to_string(infra.to_common_params(0)).unwrap()
                    ))
                    .build()
                    .unwrap(),
            )
            .body(Body::from(serde_html_form::to_string(params.clone()).unwrap()))
            .unwrap();
        assert_eq!(
            TestWithArgsRequest::from_request(req, &infra.state()).await.unwrap().params,
            params
        );
    }
}
