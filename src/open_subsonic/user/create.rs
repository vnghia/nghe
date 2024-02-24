use super::password::encrypt_password;
use crate::models::*;
use crate::open_subsonic::browsing::refresh_permissions;
use crate::{Database, OSResult};

use axum::extract::State;
use derivative::Derivative;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use nghe_proc_macros::{add_validate, wrap_subsonic_response};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[add_validate(admin = true)]
#[derive(Derivative, Default, Deserialize, PartialEq, Eq)]
#[derivative(Debug)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize)]
pub struct CreateUserBody {}

pub async fn create_user_handler(
    State(database): State<Database>,
    req: CreateUserRequest,
) -> OSResult<CreateUserResponse> {
    create_user(&database, req.params).await?;
    Ok(CreateUserBody {}.into())
}

pub async fn create_user(
    Database { pool, key }: &Database,
    params: CreateUserParams,
) -> OSResult<users::User> {
    let CreateUserParams {
        username,
        password,
        email,
        admin_role,
        download_role,
        share_role,
        ..
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
    use super::*;
    use crate::utils::test::{
        setup::setup_users_and_music_folders, user::create_username_password,
    };

    use diesel::{ExpressionMethods, QueryDsl};
    use itertools::Itertools;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_user_with_music_folders() {
        let (temp_db, _, _temp_fs, music_folders) = setup_users_and_music_folders(0, 2, &[]).await;
        let (username, password) = create_username_password();

        // should re-trigger the refreshing of music folders
        let user = create_user(
            temp_db.database(),
            CreateUserParams {
                username,
                password,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let results = user_music_folder_permissions::table
            .select(user_music_folder_permissions::music_folder_id)
            .filter(user_music_folder_permissions::user_id.eq(user.id))
            .load::<Uuid>(&mut temp_db.pool().get().await.unwrap())
            .await
            .unwrap()
            .into_iter()
            .sorted()
            .collect_vec();

        assert_eq!(
            music_folders
                .into_iter()
                .map(|music_folder| music_folder.id)
                .sorted()
                .collect_vec(),
            results
        );
    }
}
