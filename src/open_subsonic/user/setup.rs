use axum::extract::State;
use axum::Form;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::wrap_subsonic_response;
use serde::Deserialize;
use serde_with::serde_as;

use super::create::{create_user, CreateUserParams};
use crate::models::*;
use crate::{Database, OSError};

#[serde_as]
#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct SetupParams {
    pub username: String,
    #[serde_as(as = "serde_with::Bytes")]
    pub password: Vec<u8>,
    pub email: String,
}

#[wrap_subsonic_response]
#[derive(Debug)]
pub struct SetupBody {}

pub async fn setup_handler(
    State(database): State<Database>,
    Form(params): Form<SetupParams>,
) -> SetupJsonResponse {
    if users::table.count().first::<i64>(&mut database.pool.get().await?).await? != 0 {
        Err(OSError::Forbidden("access setup when there is user".into()).into())
    } else {
        create_user(
            &database,
            CreateUserParams {
                username: params.username,
                password: params.password,
                email: params.email,
                admin_role: true,
                download_role: true,
                share_role: true,
            },
        )
        .await?;
        SetupBody {}.into()
    }
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};

    use super::*;
    use crate::utils::test::user::{create_username_password, create_users};

    #[tokio::test]
    async fn test_setup_no_user() {
        let (temp_db, _) = create_users(0, 0).await;
        let (username, password) = create_username_password();

        let form = Form(SetupParams { username, password, ..Faker.fake() });

        assert!(setup_handler(temp_db.state(), form).await.is_ok());
    }

    #[tokio::test]
    async fn test_setup_with_user() {
        let (temp_db, _) = create_users(1, 1).await;
        let (username, password) = create_username_password();

        let form = Form(SetupParams { username, password, ..Faker.fake() });

        if let Some(err) = setup_handler(temp_db.state(), form).await.err() {
            assert!(matches!(
                err.0.root_cause().downcast_ref::<OSError>().unwrap(),
                OSError::Forbidden(_)
            ));
        } else {
            unreachable!("setup can not be used when there is user");
        }
    }
}
