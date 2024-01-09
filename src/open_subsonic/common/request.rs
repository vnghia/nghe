use super::super::user::password::*;
use super::super::{OSResult, OpenSubsonicError};
use crate::config::EncryptionKey;
use crate::entity::{prelude::*, *};
use crate::ServerState;

use axum::extract::{rejection::FormRejection, Form, FromRef, FromRequest, Request};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, *};
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct CommonParams {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "t")]
    pub token: String,
    #[serde(rename = "s")]
    pub salt: String,
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
        let user = params
            .validate(&state.conn, &state.config.database.encryption_key)
            .await?;
        Ok(ValidatedForm { params, user })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::utils::tests::*;

    use fake::{faker::internet::en::*, Fake, Faker};

    use nghe_proc_macros::add_validate;

    #[add_validate]
    #[derive(Debug, Default, Deserialize, PartialEq, Eq)]
    struct TestParams {}

    #[add_validate(admin = true)]
    #[derive(Debug, Default, Deserialize, PartialEq, Eq)]
    struct AdminTestParams {}

    async fn setup_db_and_user(
        username: String,
        password: &String,
        key: &EncryptionKey,
        admin_role: bool,
    ) -> TemporaryDatabase {
        let current_timestamp = std::time::SystemTime::now();
        let db = TemporaryDatabase::new_from_env().await;
        db.insert(
            user::Model {
                username: username,
                password: encrypt_password(&key, &password),
                created_at: current_timestamp.into(),
                updated_at: current_timestamp.into(),
                admin_role: admin_role,
                ..Faker.fake()
            }
            .into_active_model(),
        )
        .await
        .to_owned()
    }

    #[tokio::test]
    async fn test_validate_success() {
        let key: EncryptionKey = rand::random();

        let username: String = Username().fake();
        let password: String = Password(16..32).fake();

        let client_salt: String = Password(8..16).fake();
        let client_token = to_password_token(&password, &client_salt);

        let db = setup_db_and_user(username.clone(), &password, &key, false).await;

        assert!(TestParams {
            common: CommonParams {
                username: username.clone(),
                token: client_token,
                salt: client_salt
            }
        }
        .validate(db.get_conn(), &key)
        .await
        .is_ok());

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_validate_wrong_username() {
        let key: EncryptionKey = rand::random();

        let username: String = Username().fake();
        let wrong_username: String = Username().fake();
        let password: String = Password(16..32).fake();

        let client_salt: String = Password(8..16).fake();
        let client_token = to_password_token(&password, &client_salt);

        let db = setup_db_and_user(username.clone(), &password, &key, false).await;

        assert!(matches!(
            TestParams {
                common: CommonParams {
                    username: wrong_username.clone(),
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
    async fn test_validate_wrong_password() {
        let key: EncryptionKey = rand::random();

        let username: String = Username().fake();
        let password: String = Password(16..32).fake();
        let wrong_password: String = Password(16..32).fake();

        let client_salt: String = Password(8..16).fake();
        let client_token = to_password_token(&wrong_password, &client_salt);

        let db = setup_db_and_user(username.clone(), &password, &key, false).await;

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
        let key: EncryptionKey = rand::random();

        let username: String = Username().fake();
        let password: String = Password(16..32).fake();

        let client_salt: String = Password(8..16).fake();
        let client_token = to_password_token(&password, &client_salt);

        let db = setup_db_and_user(username.clone(), &password, &key, true).await;

        assert!(AdminTestParams {
            common: CommonParams {
                username: username.clone(),
                token: client_token,
                salt: client_salt
            }
        }
        .validate(db.get_conn(), &key)
        .await
        .is_ok());

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_validate_no_admin() {
        let key: EncryptionKey = rand::random();

        let username: String = Username().fake();
        let password: String = Password(16..32).fake();

        let client_salt: String = Password(8..16).fake();
        let client_token = to_password_token(&password, &client_salt);

        let db = setup_db_and_user(username.clone(), &password, &key, false).await;

        assert!(matches!(
            AdminTestParams {
                common: CommonParams {
                    username: username.clone(),
                    token: client_token,
                    salt: client_salt
                }
            }
            .validate(db.get_conn(), &key)
            .await,
            Err(OpenSubsonicError::Forbidden { message: _ })
        ));

        db.async_drop().await;
    }
}
