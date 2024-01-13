use super::super::user::password::*;
use crate::config::EncryptionKey;
use crate::entity::{prelude::*, *};
use crate::{OSResult, OpenSubsonicError, ServerState};

use axum::extract::{rejection::FormRejection, Form, FromRef, FromRequest, Request};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, *};
use serde::{de::DeserializeOwned, Deserialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct CommonParams {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "s")]
    pub salt: String,
    #[serde(rename = "t")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub token: MD5Token,
}

#[async_trait::async_trait]
pub trait Validate {
    fn get_common_params(&self) -> &CommonParams;

    fn need_admin(&self) -> bool;

    async fn validate(
        &self,
        conn: &DatabaseConnection,
        key: &EncryptionKey,
    ) -> OSResult<user::Model> {
        let common_params = self.get_common_params();
        let user: user::Model = match User::find()
            .filter(user::Column::Username.eq(&common_params.username))
            .one(conn)
            .await?
        {
            Some(user) => user,
            _ => return Err(OpenSubsonicError::Unauthorized { message: None }),
        };
        check_password(
            &decrypt_password(key, &user.password)?,
            &common_params.salt,
            &common_params.token,
        )?;
        if self.need_admin() && !user.admin_role {
            return Err(OpenSubsonicError::Forbidden { message: None });
        }
        Ok(user)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedForm<T> {
    pub params: T,
    pub user: user::Model,
}

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate + Send + Sync + std::fmt::Debug,
    ServerState: FromRef<S>,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = OpenSubsonicError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(params) = Form::<T>::from_request(req, state).await?;
        tracing::debug!("deserialized form {:?}", params);
        let state = ServerState::from_ref(state);
        let user = params.validate(&state.conn, &state.encryption_key).await?;
        Ok(ValidatedForm { params, user })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::user::create::create_user,
        open_subsonic::user::create::CreateUserParams,
        utils::test::user::{create_db_key_users, create_user_token},
    };

    use fake::{faker::internet::en::*, Fake};

    use nghe_proc_macros::add_validate;

    #[add_validate]
    #[derive(Debug, Default, Deserialize, PartialEq, Eq)]
    struct TestParams {}

    #[add_validate(admin = true)]
    #[derive(Debug, Default, Deserialize, PartialEq, Eq)]
    struct AdminTestParams {}

    #[tokio::test]
    async fn test_validate_success() {
        let (db, key, user_tokens) = create_db_key_users(1, 0).await;

        assert!(TestParams {
            common: CommonParams {
                username: user_tokens[0].0.username.clone(),
                salt: user_tokens[0].1.clone(),
                token: user_tokens[0].2,
            }
        }
        .validate(db.get_conn(), &key)
        .await
        .is_ok());

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_validate_wrong_username() {
        let (db, key, user_tokens) = create_db_key_users(1, 0).await;
        let wrong_username: String = Username().fake();

        assert!(matches!(
            TestParams {
                common: CommonParams {
                    username: wrong_username.clone(),
                    salt: user_tokens[0].1.clone(),
                    token: user_tokens[0].2,
                }
            }
            .validate(db.get_conn(), &key)
            .await,
            Err(OpenSubsonicError::Unauthorized { message: _ })
        ));

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_validate_wrong_password() {
        let (db, key, _) = create_db_key_users(0, 0).await;
        let (username, _, client_salt, client_token) = create_user_token();
        let wrong_password: String = Password(16..32).fake();
        let _ = create_user(
            db.get_conn(),
            &key,
            CreateUserParams {
                username: username.clone(),
                password: wrong_password,
                ..Default::default()
            },
        )
        .await;

        assert!(matches!(
            TestParams {
                common: CommonParams {
                    username: username.clone(),
                    token: client_token,
                    salt: client_salt
                }
            }
            .validate(db.get_conn(), &key)
            .await,
            Err(OpenSubsonicError::Unauthorized { message: _ })
        ));

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_validate_admin_success() {
        let (db, key, user_tokens) = create_db_key_users(1, 1).await;

        assert!(AdminTestParams {
            common: CommonParams {
                username: user_tokens[0].0.username.clone(),
                salt: user_tokens[0].1.clone(),
                token: user_tokens[0].2,
            }
        }
        .validate(db.get_conn(), &key)
        .await
        .is_ok());

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_validate_no_admin() {
        let (db, key, user_tokens) = create_db_key_users(1, 0).await;

        assert!(matches!(
            AdminTestParams {
                common: CommonParams {
                    username: user_tokens[0].0.username.clone(),
                    salt: user_tokens[0].1.clone(),
                    token: user_tokens[0].2,
                }
            }
            .validate(db.get_conn(), &key)
            .await,
            Err(OpenSubsonicError::Forbidden { message: _ })
        ));

        db.async_drop().await;
    }
}
