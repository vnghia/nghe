use super::db::TemporaryDatabase;
use crate::config::EncryptionKey;
use crate::entity::*;
use crate::open_subsonic::user::create::{create_user, CreateUserParams};
use crate::open_subsonic::user::password::{to_password_token, MD5Token};

use fake::{faker::internet::en::*, Fake};

pub fn create_user_token() -> (String, String, String, MD5Token) {
    let username: String = Username().fake();
    let password: String = Password(16..32).fake();
    let client_salt: String = Password(8..16).fake();
    let client_token = to_password_token(&password, &client_salt);
    (username, password, client_salt, client_token)
}

pub async fn create_db_key_users(
    n_user: u8,
    n_admin: u8,
) -> (
    TemporaryDatabase,
    EncryptionKey,
    Vec<(user::Model, String, MD5Token)>,
) {
    let key = rand::random();
    let db = TemporaryDatabase::new_from_env().await;

    let mut user_tokens = Vec::<(user::Model, String, MD5Token)>::default();

    for i in 0..n_user {
        let (username, password, client_salt, client_token) = create_user_token();

        let user = create_user(
            db.get_conn(),
            &key,
            CreateUserParams {
                username,
                password,
                admin_role: i < n_admin,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        user_tokens.push((user, client_salt, client_token));
    }

    (db, key, user_tokens)
}

pub async fn create_db_users(
    n_user: u8,
    n_admin: u8,
) -> (TemporaryDatabase, Vec<(user::Model, String, MD5Token)>) {
    let (db, _, user_tokens) = create_db_key_users(n_user, n_admin).await;
    (db, user_tokens)
}
