use anyhow::Result;
use axum::extract::State;
use derivative::Derivative;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde_with::serde_as;

use crate::models::*;
use crate::open_subsonic::browsing::refresh_permissions;
use crate::utils::password::encrypt_password;
use crate::Database;

#[serde_as]
#[add_validate(admin = true)]
#[derive(Derivative)]
#[derivative(Debug)]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct CreateUserParams {
    pub username: String,
    #[derivative(Debug = "ignore")]
    #[serde_as(as = "serde_with::Bytes")]
    pub password: Vec<u8>,
    pub email: String,
    pub admin_role: bool,
    pub download_role: bool,
    pub share_role: bool,
}

#[wrap_subsonic_response]
pub struct CreateUserBody {}

pub async fn create_user_handler(
    State(database): State<Database>,
    req: CreateUserRequest,
) -> CreateUserJsonResponse {
    create_user(&database, req.params).await?;
    CreateUserBody {}.into()
}

pub async fn create_user(
    Database { pool, key }: &Database,
    params: CreateUserParams,
) -> Result<users::User> {
    let CreateUserParams {
        username, password, email, admin_role, download_role, share_role, ..
    } = params;
    let password = encrypt_password(key, &password);

    let user = diesel::insert_into(users::table)
        .values(&users::NewUser {
            username: username.into(),
            password: password.into(),
            email: email.into(),
            admin_role,
            download_role,
            share_role,
        })
        .returning(users::User::as_returning())
        .get_result(&mut pool.get().await?)
        .await?;
    refresh_permissions(pool, Some(&[user.id]), None).await?;
    Ok(user)
}

#[cfg(test)]
mod tests {
    use diesel::{ExpressionMethods, QueryDsl};
    use fake::{Fake, Faker};
    use itertools::Itertools;
    use uuid::Uuid;

    use super::*;
    use crate::utils::test::setup::TestInfra;
    use crate::utils::test::user::create_username_password;

    #[tokio::test]
    async fn test_create_user_with_music_folders() {
        let test_infra = TestInfra::setup_users_and_music_folders(0, 2, &[]).await;
        let (username, password) = create_username_password();

        // should re-trigger the refreshing of music folders
        let user = create_user(
            test_infra.database(),
            CreateUserParams { username, password, ..Faker.fake() },
        )
        .await
        .unwrap();

        let results = user_music_folder_permissions::table
            .select(user_music_folder_permissions::music_folder_id)
            .filter(user_music_folder_permissions::user_id.eq(user.id))
            .load::<Uuid>(&mut test_infra.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(test_infra.music_folder_ids(..).into_iter().sorted().collect_vec(), results);
    }
}
