use super::super::user::password::*;
use crate::config::EncryptionKey;
use crate::models::*;
use crate::{DatabasePool, OSResult, OpenSubsonicError, ServerState};

use axum::extract::{rejection::FormRejection, Form, FromRef, FromRequest, Request};
use derivative::Derivative;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::{de::DeserializeOwned, Deserialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Derivative, Default, Deserialize, PartialEq, Eq)]
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

#[async_trait::async_trait]
pub trait Validate {
    fn get_common_params(&self) -> &CommonParams;

    fn need_admin(&self) -> bool;

    async fn validate(&self, pool: &DatabasePool, key: &EncryptionKey) -> OSResult<users::User> {
        let common_params = self.get_common_params();
        let user = match users::table
            .filter(users::username.eq(&common_params.username))
            .select(users::User::as_select())
            .first(&mut pool.get().await?)
            .await
        {
            Ok(user) => user,
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
    pub user: users::User,
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
        let user = params
            .validate(&state.database.pool, &state.database.key)
            .await?;
        Ok(ValidatedForm { params, user })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::user::create::create_user,
        open_subsonic::user::create::CreateUserParams,
        utils::test::user::{create_user_token, create_users},
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
        let (db, mut user_tokens) = create_users(1, 0).await;

        let user_token = user_tokens.remove(0);
        assert!(TestParams {
            common: CommonParams {
                username: user_token.0.username,
                salt: user_token.1,
                token: user_token.2,
            }
        }
        .validate(db.get_pool(), db.get_key())
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn test_validate_wrong_username() {
        let (db, mut user_tokens) = create_users(1, 0).await;
        let wrong_username: String = Username().fake();

        let user_token = user_tokens.remove(0);
        assert!(matches!(
            TestParams {
                common: CommonParams {
                    username: wrong_username,
                    salt: user_token.1,
                    token: user_token.2,
                }
            }
            .validate(db.get_pool(), db.get_key())
            .await,
            Err(OpenSubsonicError::Unauthorized { message: _ })
        ));
    }

    #[tokio::test]
    async fn test_validate_wrong_password() {
        let (db, _) = create_users(0, 0).await;
        let (username, _, client_salt, client_token) = create_user_token();
        let wrong_password = Password(16..32).fake::<String>().into_bytes();
        let _ = create_user(
            db.get_pool(),
            db.get_key(),
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
                    username,
                    token: client_token,
                    salt: client_salt
                }
            }
            .validate(db.get_pool(), db.get_key())
            .await,
            Err(OpenSubsonicError::Unauthorized { message: _ })
        ));
    }

    #[tokio::test]
    async fn test_validate_admin_success() {
        let (db, mut user_tokens) = create_users(1, 1).await;

        let user_token = user_tokens.remove(0);
        assert!(AdminTestParams {
            common: CommonParams {
                username: user_token.0.username,
                salt: user_token.1,
                token: user_token.2,
            }
        }
        .validate(db.get_pool(), db.get_key())
        .await
        .is_ok());
    }

    #[tokio::test]
    async fn test_validate_no_admin() {
        let (db, mut user_tokens) = create_users(1, 0).await;

        let user_token = user_tokens.remove(0);
        assert!(matches!(
            AdminTestParams {
                common: CommonParams {
                    username: user_token.0.username,
                    salt: user_token.1,
                    token: user_token.2,
                }
            }
            .validate(db.get_pool(), db.get_key())
            .await,
            Err(OpenSubsonicError::Forbidden { message: _ })
        ));
    }
}
