use super::db::TemporaryDatabase;
use crate::config::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::user::create::{create_user, CreateUserParams};
use crate::open_subsonic::user::password::{to_password_token, MD5Token};

use fake::{faker::internet::en::*, Fake};
use futures::stream::{self, StreamExt};

pub fn create_user_token() -> (String, Vec<u8>, Vec<u8>, MD5Token) {
    let username: String = Username().fake();
    let password = Password(16..32).fake::<String>().into_bytes();
    let client_salt = Password(8..16).fake::<String>().into_bytes();
    let client_token = to_password_token(&password, &client_salt);
    (username, password, client_salt, client_token)
}

pub async fn create_db_key_users(
    n_user: usize,
    n_admin: usize,
) -> (
    TemporaryDatabase,
    EncryptionKey,
    Vec<(users::User, Vec<u8>, MD5Token)>,
) {
    let key = rand::random();
    let db = TemporaryDatabase::new_from_env().await;

    let user_tokens = stream::iter(0..n_user)
        .zip(stream::repeat(db.get_pool()))
        .then(|(i, pool)| async move {
            let (username, password, client_salt, client_token) = create_user_token();
            let user = create_user(
                pool,
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
            (user, client_salt, client_token)
        })
        .collect::<Vec<_>>()
        .await;

    (db, key, user_tokens)
}
