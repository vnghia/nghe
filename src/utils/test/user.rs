use super::database::TemporaryDatabase;
use crate::database::EncryptionKey;
use crate::models::*;
use crate::open_subsonic::tests::CommonParams;
use crate::open_subsonic::user::password::{decrypt_password, to_password_token, MD5Token};
use crate::open_subsonic::user::tests::{create_user, CreateUserParams};

use fake::{faker::internet::en::*, Fake, Faker};
use futures::stream::{self, StreamExt};

pub fn create_username_password() -> (String, Vec<u8>) {
    let username: String = Username().fake();
    let password = Password(16..32).fake::<String>().into_bytes();
    (username, password)
}

pub fn create_password_token(password: &[u8]) -> (Vec<u8>, MD5Token) {
    let client_salt = Password(8..16).fake::<String>().into_bytes();
    let client_token = to_password_token(password, &client_salt);
    (client_salt, client_token)
}

pub async fn create_users(n_user: usize, n_admin: usize) -> (TemporaryDatabase, Vec<users::User>) {
    let temp_db = TemporaryDatabase::new_from_env().await;

    let users = stream::iter(0..n_user)
        .zip(stream::repeat(temp_db.database()))
        .then(|(i, database)| async move {
            let (username, password) = create_username_password();
            create_user(
                database,
                CreateUserParams {
                    username,
                    password,
                    admin_role: i < n_admin,
                    ..Faker.fake()
                },
            )
            .await
            .unwrap()
        })
        .collect::<Vec<_>>()
        .await;

    (temp_db, users)
}

impl users::User {
    pub fn to_common_params(&self, key: &EncryptionKey) -> CommonParams {
        let decrypted_password = decrypt_password(key, &self.password).unwrap();
        let (salt, token) = create_password_token(&decrypted_password);
        CommonParams {
            username: self.username.to_owned(),
            salt,
            token,
        }
    }
}
