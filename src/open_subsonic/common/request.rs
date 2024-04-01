use std::marker::PhantomData;

use anyhow::Result;
use axum::extract::{FromRef, FromRequest, Request};
use axum_extra::extract::Form;
use derivative::Derivative;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_with::serde_as;
use uuid::Uuid;

use super::error::ServerError;
use crate::models::*;
use crate::utils::password::*;
use crate::{Database, OSError};

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

    async fn validate<const REQUIRED_ROLE: users::Role>(
        &self,
        Database { pool, key }: &Database,
    ) -> Result<Uuid> {
        let common_params = self.common();
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
            &decrypt_password(key, &user_password)?,
            &common_params.salt,
            &common_params.token,
        )?;
        if REQUIRED_ROLE > user_role {
            anyhow::bail!(OSError::Forbidden("access admin endpoint".into()));
        }
        Ok(user_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedForm<R, P, const REQUIRED_ROLE: users::Role> {
    pub params: P,
    pub user_id: Uuid,
    pub phantom: PhantomData<R>,
}

#[async_trait::async_trait]
impl<R, P, const REQUIRED_ROLE: users::Role, S> FromRequest<S>
    for ValidatedForm<R, P, REQUIRED_ROLE>
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
        let user_id = request_params.validate::<REQUIRED_ROLE>(&database).await?;
        Ok(ValidatedForm { params: request_params.params(), user_id, phantom: PhantomData })
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::*;
    use fake::Fake;
    use nghe_proc_macros::add_validate;

    use super::*;
    use crate::utils::test::Infra;

    #[add_validate]
    struct TestParams {}

    #[tokio::test]
    async fn test_validate_success() {
        let infra = Infra::new().await.add_user(None).await;
        assert!(
            TestParams {}
                .to_validate(infra.to_common_params(0))
                .validate::<{ users::Role::const_default() }>(infra.database())
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validate_wrong_username() {
        let infra = Infra::new().await.add_user(None).await;
        let wrong_username: String = Username().fake();
        assert!(matches!(
            TestParams {}
                .to_validate(CommonParams { username: wrong_username, ..infra.to_common_params(0) })
                .validate::<{ users::Role::const_default() }>(infra.database())
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

        let username = infra.users[0].username.clone();
        let client_salt = Password(8..16).fake::<String>().into_bytes();
        let client_token =
            to_password_token(&Password(16..32).fake::<String>().into_bytes(), &client_salt);

        assert!(matches!(
            TestParams {}
                .to_validate(CommonParams { username, salt: client_salt, token: client_token })
                .validate::<{ users::Role::const_default() }>(infra.database())
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
            TestParams {}
                .to_validate(infra.to_common_params(0))
                .validate::<{ users::Role { admin_role: true, ..users::Role::const_default() } }>(
                    infra.database()
                )
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_validate_no_admin() {
        let infra = Infra::new().await.add_user(None).await;
        assert!(matches!(
            TestParams {}
                .to_validate(infra.to_common_params(0))
                .validate::<{ users::Role { admin_role: true, ..users::Role::const_default() } }>(
                    infra.database()
                )
                .await
                .unwrap_err()
                .root_cause()
                .downcast_ref::<OSError>()
                .unwrap(),
            OSError::Forbidden(_)
        ));
    }
}
