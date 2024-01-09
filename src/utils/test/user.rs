use crate::config::EncryptionKey;
use crate::open_subsonic::user::password::to_password_token;

use fake::{faker::internet::en::*, Fake};

pub fn create_key_user_token() -> (EncryptionKey, String, String, String, String) {
    let key: EncryptionKey = rand::random();
    let username: String = Username().fake();
    let password: String = Password(16..32).fake();
    let client_salt: String = Password(8..16).fake();
    let client_token = to_password_token(&password, &client_salt);
    (key, username, password, client_salt, client_token)
}
