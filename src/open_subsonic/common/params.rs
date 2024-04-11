use std::marker::PhantomData;

use anyhow::Result;
use axum::extract::{FromRef, FromRequest, Request};
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

async fn validate<P: AsRef<CommonParams>, const REQUIRED_ROLE: users::Role>(
    Database { pool, key }: &Database,
    common_params: P,
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
    R: DeserializeOwned + Send + Sync + AsRef<CommonParams> + Into<P>,
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(request_params) = Form::<R>::from_request(req, state)
            .await
            .map_err(std::convert::Into::<OSError>::into)?;
        let database = Database::from_ref(state);
        let (user_id, user_role) = validate::<_, REQUIRED_ROLE>(&database, &request_params).await?;
        Ok(ValidatedForm {
            params: request_params.into(),
            user_id,
            user_role,
            phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::*;
    use fake::Fake;
    use nghe_proc_macros::add_common_convert;
    use nghe_types::params::{to_password_token, WithCommon};

    use super::*;
    use crate::utils::test::Infra;

    #[add_common_convert]
    struct TestParams {}

    #[tokio::test]
    async fn test_validate_success() {
        let infra = Infra::new().await.add_user(None).await;
        assert!(
            validate::<_, { users::Role::const_default() }>(
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
            validate::<_, { users::Role::const_default() }>(
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
            validate::<_, { users::Role::const_default() }>(
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
            validate::<_, { users::Role { admin_role: true, ..users::Role::const_default() } }>(
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
            validate::<_, { users::Role { admin_role: true, ..users::Role::const_default() } }>(
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
}
