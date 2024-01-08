use super::super::user::password::*;
use super::super::{OSResult, OpenSubsonicError};
use crate::entity::{prelude::*, *};

use libaes::Cipher;
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

pub trait AuthenticatedForm {
    fn get_common_params(&self) -> CommonParams;

    #[allow(async_fn_in_trait)]
    async fn check_authentication(
        &self,
        conn: &DatabaseConnection,
        cipher: &Cipher,
    ) -> OSResult<user::Model> {
        let common_params = self.get_common_params();
        let current_user: user::Model = match User::find()
            .filter(user::Column::Username.eq(common_params.username))
            .one(conn)
            .await?
        {
            Some(result) => result,
            _ => return Err(OpenSubsonicError::Unauthorized { message: None }),
        };
        check_password(
            decrypt_password(cipher, &current_user.password)?,
            &common_params.salt,
            &common_params.token,
        )?;
        Ok(current_user)
    }
}
