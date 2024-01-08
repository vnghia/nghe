use super::super::user::password::*;
use super::super::{OSResult, OpenSubsonicError};
use crate::config::EncryptionKey;
use crate::entity::{prelude::*, *};
use crate::ServerState;

use axum::extract::FromRef;
use axum::extract::{Form, FromRequest, Request};
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
        Ok(user)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedForm<T> {
    pub form: T,
    pub user: user::Model,
}

#[async_trait::async_trait]
impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate + Send + Sync,
    ServerState: FromRef<S>,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = OpenSubsonicError>,
{
    type Rejection = OpenSubsonicError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(form) = Form::<T>::from_request(req, state).await?;
        let state = ServerState::from_ref(state);
        let user = form
            .validate(&state.conn, &state.config.database.encryption_key)
            .await?;
        Ok(ValidatedForm { form, user })
    }
}
