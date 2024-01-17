use super::create::{create_user, CreateUserParams};
use crate::entity::prelude::*;
use crate::{OSResult, OpenSubsonicError, ServerState};

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

pub async fn setup_handler(
    State(state): State<ServerState>,
    Form(params): Form<SetupParams>,
) -> OSResult<SetupResponse> {
    if User::find().count(&state.conn).await? != 0 {
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

#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::FreeEmail, Fake};

    use super::*;
    use crate::utils::test::{
        db::TemporaryDatabase, state::setup_state, user::create_db_key_users,
        user::create_user_token,
    };

    #[tokio::test]
    async fn test_setup_no_user() {
        let db = TemporaryDatabase::new_from_env().await;
        let key = rand::random();
        let (username, password, _, _) = create_user_token();

        let state = setup_state(db.get_conn(), key);
        let form = Form(SetupParams {
            username,
            password,
            email: FreeEmail().fake(),
        });

        assert_eq!(
            setup_handler(state, form).await.unwrap().0,
            SetupBody::default().into()
        );

        db.async_drop().await;
    }

    #[tokio::test]
    async fn test_setup_with_user() {
        let (db, key, _) = create_db_key_users(1, 1).await;
        let (username, password, _, _) = create_user_token();

        let state = setup_state(db.get_conn(), key);
        let form = Form(SetupParams {
            username,
            password,
            email: FreeEmail().fake(),
        });

        assert!(matches!(
            setup_handler(state, form).await,
            Err(OpenSubsonicError::Forbidden { message: _ })
        ));

        db.async_drop().await;
    }
}
