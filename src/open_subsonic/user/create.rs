use super::password::encrypt_password;
use crate::config::EncryptionKey;
use crate::entity::{prelude::*, *};
use crate::open_subsonic::browsing::refresh_user_music_folders_all_folders;
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
) -> OSResult<user::Model> {
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
    let user = User::insert(user)
        .exec_with_returning(conn)
        .await
        .map_err(|e| crate::OpenSubsonicError::Generic { source: e.into() })?;
    refresh_user_music_folders_all_folders(conn, &[user.id]).await?;
    Ok(user)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        open_subsonic::browsing::test::setup_user_and_music_folders,
        utils::test::user::create_user_token,
    };

    use itertools::Itertools;

    #[tokio::test]
    async fn test_create_user_with_music_folders() {
        let (db, key, _, _temp_fs, music_folders, _) =
            setup_user_and_music_folders(0, 2, &[]).await;
        let (username, password, _, _) = create_user_token();

        let user = create_user(
            db.get_conn(),
            &key,
            CreateUserParams {
                username: username.clone(),
                password,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let results = UserMusicFolder::find()
            .filter(user_music_folder::Column::UserId.eq(user.id))
            .all(db.get_conn())
            .await
            .unwrap()
            .into_iter()
            .sorted_by_key(|user_music_folder| user_music_folder.music_folder_id)
            .collect::<Vec<_>>();

        assert_eq!(
            music_folders
                .into_iter()
                .sorted_by_key(|music_folder| music_folder.id)
                .map(|music_folder| user_music_folder::Model {
                    user_id: user.id,
                    music_folder_id: music_folder.id,
                    allow: true
                })
                .collect::<Vec<_>>(),
            results
        );

        db.async_drop().await;
    }
}
