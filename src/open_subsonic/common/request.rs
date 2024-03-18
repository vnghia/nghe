use super::error::ServerError;
use crate::models::*;
use crate::utils::password::*;
use crate::{Database, OSError};

use anyhow::Result;
use axum::extract::{FromRef, FromRequest, Request};
use axum_extra::extract::Form;
use derivative::Derivative;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde::{de::DeserializeOwned, Deserialize};
use serde_with::serde_as;
use std::marker::PhantomData;
use uuid::Uuid;

#[serde_as]
#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
#[cfg_attr(test, derive(fake::Dummy))]
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

#[async_trait::async_trait]
pub trait Validate<P> {
    fn common(&self) -> &CommonParams;
    fn params(self) -> P;

    async fn validate<const A: bool>(&self, Database { pool, key }: &Database) -> Result<Uuid> {
        let common_params = self.common();
        let (user_id, user_password, user_is_admin) = match users::table
            .filter(users::username.eq(&common_params.username))
            .select((users::id, users::password, users::admin_role))
            .first::<(Uuid, Vec<u8>, bool)>(&mut pool.get().await?)
            .await
        {
            Ok(res) => res,
            _ => anyhow::bail!(OSError::Unauthorized),
        };

        check_password(
            &decrypt_password(key, &user_password)?,
            &common_params.salt,
            &common_params.token,
        )?;
        if A && !user_is_admin {
            anyhow::bail!(OSError::Forbidden("access admin endpoint".into()));
        }
        Ok(user_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedForm<R, P, const A: bool> {
    pub params: P,
    pub user_id: Uuid,
    pub phantom: PhantomData<R>,
}

#[async_trait::async_trait]
impl<R, P, const A: bool, S> FromRequest<S> for ValidatedForm<R, P, A>
where
    R: DeserializeOwned + Send + Sync + Validate<P>,
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(request_params) = Form::<R>::from_request(req, state)
            .await
            .map_err(std::convert::Into::<OSError>::into)?;
        let database = Database::from_ref(state);
        let user_id = request_params.validate::<A>(&database).await?;
        Ok(ValidatedForm {
            params: request_params.params(),
            user_id,
            phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::user::test::{create_user, CreateUserParams},
        utils::test::user::{create_password_token, create_username_password, create_users},
    };

    use fake::{faker::internet::en::*, Fake, Faker};
    use nghe_proc_macros::add_validate;

    #[add_validate]
    struct TestParams {}

    #[tokio::test]
    async fn test_validate_success() {
        let (temp_db, users) = create_users(1, 0).await;
        assert!(TestParams {}
            .to_validate(users[0].to_common_params(temp_db.key()))
            .validate::<false>(temp_db.database())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_validate_wrong_username() {
        let (temp_db, users) = create_users(1, 0).await;
        let wrong_username: String = Username().fake();
        assert!(matches!(
            TestParams {}
                .to_validate(CommonParams {
                    username: wrong_username,
                    ..users[0].to_common_params(temp_db.key())
                })
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
            TestParams {}
                .to_validate(CommonParams {
                    username,
                    salt: client_salt,
                    token: client_token
                })
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
        assert!(TestParams {}
            .to_validate(users[0].to_common_params(temp_db.key()))
            .validate::<true>(temp_db.database())
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_validate_no_admin() {
        let (temp_db, users) = create_users(1, 0).await;
        assert!(matches!(
            TestParams {}
                .to_validate(users[0].to_common_params(temp_db.key()))
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
