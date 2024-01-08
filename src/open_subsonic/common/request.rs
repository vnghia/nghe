use super::super::user::password::*;
use super::super::{OSResult, OpenSubsonicError};
use crate::config::EncryptionKey;
use crate::entity::{prelude::*, *};

use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, *};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct CommonParams {
    #[serde(rename = "u")]
    pub username: String,
    #[serde(rename = "t")]
    pub token: String,
    #[serde(rename = "s")]
    pub salt: String,
}

pub trait Validate {
    fn get_common_params(&self) -> CommonParams;

    #[allow(async_fn_in_trait)]
    async fn validate(
        &self,
        conn: &DatabaseConnection,
        key: &EncryptionKey,
    ) -> OSResult<user::Model> {
        let common_params = self.get_common_params();
        let user: user::Model = match User::find()
            .filter(user::Column::Username.eq(common_params.username))
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
