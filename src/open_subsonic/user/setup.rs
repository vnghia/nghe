use super::create::{create_user, CreateUserParams};
use crate::models::*;
use crate::{OSResult, OpenSubsonicError, ServerState};

use axum::extract::State;
use axum::Form;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::wrap_subsonic_response;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SetupParams {
    pub username: String,
    #[serde_as(as = "serde_with::Bytes")]
    pub password: Vec<u8>,
    pub email: String,
}

#[wrap_subsonic_response]
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct SetupBody {}

pub async fn setup_handler(
    State(state): State<ServerState>,
    Form(params): Form<SetupParams>,
) -> OSResult<SetupResponse> {
    if users::table
        .count()
        .first::<i64>(&mut state.database.pool.get().await?)
        .await?
        != 0
    {
        return Err(OpenSubsonicError::Forbidden {
            message: Some("setup can only be used when there is no user".into()),
        });
    }
    create_user(
        &state.database.pool,
        &state.database.key,
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
    use fake::{Fake, Faker};

    use super::*;
    use crate::utils::test::{
        state::setup_state,
        user::{create_user_token, create_users},
    };

    #[tokio::test]
    async fn test_setup_no_user() {
        let (db, _) = create_users(0, 0).await;
        let (username, password, _, _) = create_user_token();

        let state = setup_state(&db);
        let form = Form(SetupParams {
            username,
            password,
            ..Faker.fake()
        });

        assert_eq!(
            setup_handler(state, form).await.unwrap().0,
            SetupBody::default().into()
        );
    }

    #[tokio::test]
    async fn test_setup_with_user() {
        let (db, _) = create_users(1, 1).await;
        let (username, password, _, _) = create_user_token();

        let state = setup_state(&db);
        let form = Form(SetupParams {
            username,
            password,
            ..Faker.fake()
        });

        assert!(matches!(
            setup_handler(state, form).await,
            Err(OpenSubsonicError::Forbidden { message: _ })
        ));
    }
}
