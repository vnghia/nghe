use axum::extract::State;
use axum::Form;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::add_axum_response;
use nghe_types::user::create_user::CreateUserParams;

use super::create_user::create_user;
use crate::models::*;
use crate::{Database, OSError};

add_axum_response!(SetupBody);

pub async fn setup_handler(
    State(database): State<Database>,
    Form(params): Form<SetupParams>,
) -> SetupJsonResponse {
    if users::table.count().first::<i64>(&mut database.pool.get().await?).await? != 0 {
        Err(OSError::Forbidden("access setup when there is user".into()).into())
    } else {
        create_user(
            &database,
            &CreateUserParams {
                username: params.username,
                password: params.password,
                email: params.email,
                admin_role: true,
                stream_role: true,
                download_role: true,
                share_role: true,
                allow: true,
            },
        )
        .await?;
        Ok(axum::Json(SetupBody {}.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test::{Infra, User};

    #[tokio::test]
    async fn test_setup_no_user() {
        let infra = Infra::new().await;
        let CreateUserParams { username, password, .. } = User::fake(None).into();
        let form = Form(SetupParams { username, password, email: Default::default() });
        assert!(setup_handler(infra.state(), form).await.is_ok());
    }

    #[tokio::test]
    async fn test_setup_with_user() {
        let infra = Infra::new().await.add_user(None).await;
        let CreateUserParams { username, password, .. } = User::fake(None).into();
        let form = Form(SetupParams { username, password, email: Default::default() });
        if let Some(err) = setup_handler(infra.state(), form).await.err() {
            assert!(matches!(
                err.0.root_cause().downcast_ref::<OSError>().unwrap(),
                OSError::Forbidden(_)
            ));
        } else {
            unreachable!("setup can not be used when there is user");
        }
    }
}
