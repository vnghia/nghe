use super::super::OSResult;
use super::create::{create_user, CreateUserParams};
use crate::entity::*;
use crate::{OpenSubsonicError, ServerState};

use axum::extract::State;
use axum::Form;
use nghe_proc_macros::wrap_subsonic_response;
use sea_orm::{EntityTrait, *};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetupParams {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[wrap_subsonic_response]
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct SetupBody {}

#[axum::debug_handler]
pub async fn setup_handler(
    State(state): State<ServerState>,
    Form(params): Form<SetupParams>,
) -> OSResult<SetupResponse> {
    if user::Entity::find().count(&state.conn).await? != 0 {
        return Err(OpenSubsonicError::Forbidden {
            message: Some("setup can only be used when there is no user".to_owned()),
        });
    }
    create_user(
        &state.conn,
        &state.encryption_key,
        CreateUserParams {
            username: params.username,
            password: params.password,
            email: params.email,
            admin_role: true,
            download_role: true,
            share_role: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(SetupBody::default().into())
}
