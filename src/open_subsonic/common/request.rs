use super::super::user::password::*;
use super::error::ServerError;
use crate::models::*;
use crate::{Database, OSError};

use anyhow::Result;
use axum::extract::{FromRef, FromRequest, Request};
use axum_extra::extract::Form;
use derivative::Derivative;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{de::DeserializeOwned, Deserialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
pub struct CommonParams {
    #[serde(rename = "u")]
    pub username: String,
    #[derivative(Debug = "ignore")]
    #[serde(rename = "s")]
    #[serde_as(as = "serde_with::Bytes")]
    pub salt: Vec<u8>,
    #[derivative(Debug = "ignore")]
    #[serde(rename = "t")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub token: MD5Token,
}

#[derive(Debug, Deserialize)]
pub struct RequestParams<T> {
    #[serde(flatten)]
    #[serde(rename = "camelCase")]
    pub params: T,
    #[serde(flatten)]
    pub common: CommonParams,
}

impl<T> RequestParams<T> {
    pub async fn validate<const A: bool>(
        &self,
        Database { pool, key }: &Database,
    ) -> Result<users::User> {
        let common_params = &self.common;
        let user = match users::table
            .filter(users::username.eq(&common_params.username))
            .select(users::User::as_select())
            .first(&mut pool.get().await?)
            .await
        {
            Ok(user) => user,
            _ => anyhow::bail!(OSError::Unauthorized),
        };

        check_password(
            &decrypt_password(key, &user.password)?,
            &common_params.salt,
            &common_params.token,
        )?;
        if A && !user.admin_role {
            anyhow::bail!(OSError::Forbidden("access admin endpoint".into()));
        }
        Ok(user)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedForm<T, const A: bool> {
    pub params: T,
    pub user: users::User,
}

#[async_trait::async_trait]
impl<T, const A: bool, S> FromRequest<S> for ValidatedForm<T, A>
where
    RequestParams<T>: DeserializeOwned + Send + Sync,
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(request_params) = Form::<RequestParams<T>>::from_request(req, state)
            .await
            .map_err(std::convert::Into::<OSError>::into)?;
        let database = Database::from_ref(state);
        let user = request_params.validate::<A>(&database).await?;
        Ok(ValidatedForm {
            params: request_params.params,
            user,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::user::tests::{create_user, CreateUserParams},
        utils::test::user::{create_password_token, create_username_password, create_users},
    };

    use fake::{faker::internet::en::*, Fake, Faker};

    struct TestParams {}

    #[tokio::test]
    async fn test_validate_success() {
        let (temp_db, users) = create_users(1, 0).await;
        assert!(RequestParams::<TestParams> {
            params: TestParams {},
            common: users[0].to_common_params(temp_db.key())
        }
        .validate::<false>(temp_db.database())
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn test_validate_wrong_username() {
        let (temp_db, users) = create_users(1, 0).await;
        let wrong_username: String = Username().fake();
        assert!(matches!(
            RequestParams::<TestParams> {
                params: TestParams {},
                common: CommonParams {
                    username: wrong_username,
                    ..users[0].to_common_params(temp_db.key())
                }
            }
            .validate::<false>(temp_db.database())
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
        let (temp_db, _) = create_users(0, 0).await;
        let (username, password) = create_username_password();
        let wrong_password = Password(16..32).fake::<String>().into_bytes();
        let (client_salt, client_token) = create_password_token(&wrong_password);
        let _ = create_user(
            temp_db.database(),
            CreateUserParams {
                username: username.clone(),
                password,
                ..Faker.fake()
            },
        )
        .await;

        assert!(matches!(
            RequestParams::<TestParams> {
                params: TestParams {},
                common: CommonParams {
                    username,
                    salt: client_salt,
                    token: client_token
                }
            }
            .validate::<false>(temp_db.database())
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
        let (temp_db, users) = create_users(1, 1).await;
        assert!(RequestParams::<TestParams> {
            params: TestParams {},
            common: users[0].to_common_params(temp_db.key())
        }
        .validate::<true>(temp_db.database())
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn test_validate_no_admin() {
        let (temp_db, users) = create_users(1, 0).await;
        assert!(matches!(
            RequestParams::<TestParams> {
                params: TestParams {},
                common: users[0].to_common_params(temp_db.key())
            }
            .validate::<true>(temp_db.database())
            .await
            .unwrap_err()
            .root_cause()
            .downcast_ref::<OSError>()
            .unwrap(),
            OSError::Forbidden(_)
        ));
    }
}
