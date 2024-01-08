use concat_string::concat_string;
use libaes::Cipher;

use super::super::{OSResult, OpenSubsonicError};
use crate::config::EncryptionKey;

const IV_LEN: usize = 16;

pub fn encrypt_password(key: &EncryptionKey, data: &String) -> Vec<u8> {
    let plain_text = data.as_bytes();
    let iv: [u8; IV_LEN] = rand::random();
    [
        iv.as_slice(),
        Cipher::new_128(key).cbc_encrypt(&iv, plain_text).as_slice(),
    ]
    .concat()
}

pub fn decrypt_password(key: &EncryptionKey, data: &Vec<u8>) -> OSResult<String> {
    let cipher_text = &data[IV_LEN..];
    let iv = &data[..IV_LEN];
    match String::from_utf8(Cipher::new_128(key).cbc_decrypt(iv, cipher_text)) {
        Ok(plain_text) => Ok(plain_text),
        Err(_) => Err(OpenSubsonicError::BadRequest {
            message: Some("can not decrypt password".to_owned()),
        }),
    }
}

pub fn to_password_token(password: &String, client_salt: &String) -> String {
    hex::encode::<[u8; 16]>(md5::compute(concat_string!(password, client_salt)).into())
}

pub fn check_password(
    password: &String,
    client_salt: &String,
    client_token: &String,
) -> OSResult<()> {
    let password_token = to_password_token(password, client_salt);
    if &password_token == client_token {
        Ok(())
    } else {
        Err(OpenSubsonicError::Unauthorized { message: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fake::{faker::internet::en::Password, Fake};

    #[test]
    fn test_roundtrip_password() {
        let key: EncryptionKey = rand::random();
        let password: String = Password(16..32).fake();
        assert_eq!(
            password,
            decrypt_password(&key, &encrypt_password(&key, &password)).unwrap()
        )
    }

    #[test]
    fn test_to_password_token() {
        assert_eq!(
            "26719a1196d2a940705a59634eb18eab",
            to_password_token(&"sesame".to_owned(), &"c19b2d".to_owned())
        )
    }

    #[test]
    fn test_check_password_success() {
        let password: String = Password(16..32).fake();
        let client_salt: String = Password(8..16).fake();
        let client_token = to_password_token(&password, &client_salt);
        assert!(check_password(&password, &client_salt, &client_token).is_ok())
    }

    #[test]
    fn test_check_password_failed() {
        let password: String = Password(16..32).fake();
        let client_salt: String = Password(8..16).fake();
        let wrong_client_salt = Password(8..16).fake();
        let client_token = to_password_token(&password, &client_salt);
        assert!(check_password(&password, &wrong_client_salt, &client_token).is_err())
    }
}
