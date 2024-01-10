use super::create::{create_user, CreateUserParams};
use crate::entity::*;
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

#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::FreeEmail, Fake};

    use super::*;
    use crate::utils::test::{db::TemporaryDatabase, user::create_key_user_token};

    #[tokio::test]
    async fn test_setup_no_user() {
        let db = TemporaryDatabase::new_from_env().await;
        let (key, username, password, _, _) = create_key_user_token();

        let state = State(ServerState {
            conn: db.get_conn().clone(),
            encryption_key: key,
        });
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
        let db = TemporaryDatabase::new_from_env().await;
        let (_, current_username, current_password, _, _) = create_key_user_token();
        let (key, username, password, _, _) = create_key_user_token();

        let state = State(ServerState {
            conn: db.get_conn().clone(),
            encryption_key: key,
        });
        let form = Form(SetupParams {
            username,
            password,
            email: FreeEmail().fake(),
        });

        create_user(
            &state.conn,
            &state.encryption_key,
            CreateUserParams {
                username: current_username,
                password: current_password,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert!(matches!(
            setup_handler(state, form).await,
            Err(OpenSubsonicError::Forbidden { message: _ })
        ));

        db.async_drop().await;
    }
}
