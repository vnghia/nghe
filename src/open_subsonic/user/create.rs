use super::password::encrypt_password;
use crate::config::EncryptionKey;
use crate::entity::{prelude::*, *};
use crate::{OSResult, ServerState};

use axum::extract::State;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use sea_orm::DatabaseConnection;
use sea_orm::{EntityTrait, *};
use serde::{Deserialize, Serialize};

#[add_validate(admin = true)]
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserParams {
    pub username: String,
    pub password: String,
    pub email: String,
    pub admin_role: bool,
    pub download_role: bool,
    pub share_role: bool,
}

#[wrap_subsonic_response]
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct CreateUserBody {}

#[axum::debug_handler]
pub async fn create_user_handler(
    State(state): State<ServerState>,
    req: CreateUserRequest,
) -> OSResult<CreateUserResponse> {
    create_user(&state.conn, &state.encryption_key, req.params).await?;
    Ok(CreateUserBody::default().into())
}

pub async fn create_user(
    conn: &DatabaseConnection,
    key: &EncryptionKey,
    params: CreateUserParams,
) -> OSResult<()> {
    let password = encrypt_password(key, &params.password);
    let user = user::ActiveModel {
        username: ActiveValue::Set(params.username),
        password: ActiveValue::Set(password),
        email: ActiveValue::Set(params.email),
        admin_role: ActiveValue::Set(params.admin_role),
        download_role: ActiveValue::Set(params.download_role),
        share_role: ActiveValue::Set(params.share_role),
        ..Default::default()
    };
    User::insert(user).exec(conn).await?;
    Ok(())
}
