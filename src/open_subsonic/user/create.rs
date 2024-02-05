use super::password::encrypt_password;
use crate::config::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::browsing::refresh_permissions;
use crate::{DatabasePool, OSResult, ServerState};

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
#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct CreateUserBody {}

pub async fn create_user_handler(
    State(state): State<ServerState>,
    req: CreateUserRequest,
) -> OSResult<CreateUserResponse> {
    create_user(&state.database.pool, &state.database.key, req.params).await?;
    Ok(CreateUserBody::default().into())
}

pub async fn create_user(
    pool: &DatabasePool,
    key: &EncryptionKey,
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
    use crate::{
        open_subsonic::browsing::test::setup_user_and_music_folders,
        utils::test::user::create_user_token,
    };

    use diesel::{ExpressionMethods, QueryDsl};
    use itertools::Itertools;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_user_with_music_folders() {
        let (db, key, _, _temp_fs, music_folders, _) =
            setup_user_and_music_folders(0, 2, &[]).await;
        let (username, password, _, _) = create_user_token();

        let user = create_user(
            db.get_pool(),
            &key,
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
            .load::<Uuid>(&mut db.get_pool().get().await.unwrap())
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
