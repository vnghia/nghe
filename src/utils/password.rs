use anyhow::Result;
use libaes::Cipher;
use nghe_types::params::{to_password_token, MD5Token};

use crate::database::EncryptionKey;
use crate::OSError;

const IV_LEN: usize = 16;

pub fn encrypt_password(key: &EncryptionKey, data: impl AsRef<[u8]>) -> Vec<u8> {
    let data = data.as_ref();

    let iv: [u8; IV_LEN] = rand::random();
    [iv.as_slice(), Cipher::new_128(key).cbc_encrypt(&iv, data).as_slice()].concat()
}

pub fn decrypt_password(key: &EncryptionKey, data: impl AsRef<[u8]>) -> Result<Vec<u8>> {
    let data = data.as_ref();

    let cipher_text = &data[IV_LEN..];
    let iv = &data[..IV_LEN];
    Ok(Cipher::new_128(key).cbc_decrypt(iv, cipher_text))
}

pub fn check_password(
    password: impl AsRef<[u8]>,
    salt: impl AsRef<[u8]>,
    token: &MD5Token,
) -> Result<()> {
    let password = password.as_ref();
    let salt = salt.as_ref();

    let password_token = to_password_token(password, salt);
    if &password_token == token { Ok(()) } else { anyhow::bail!(OSError::Unauthorized) }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::Password;
    use fake::Fake;
    use serde::Deserialize;
    use serde_json::json;
    use serde_with::serde_as;

    use super::*;

    #[test]
    fn test_roundtrip_password() {
        let key: EncryptionKey = rand::random();
        let password = Password(16..32).fake::<String>().into_bytes();
        assert_eq!(password, decrypt_password(&key, encrypt_password(&key, &password)).unwrap())
    }

    #[test]
    fn test_to_password_token() {
        #[serde_as]
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct TestBytes(#[serde_as(as = "serde_with::hex::Hex")] MD5Token);

        assert_eq!(
            serde_json::from_value::<TestBytes>(json!("26719a1196d2a940705a59634eb18eab")).unwrap(),
            TestBytes(to_password_token(b"sesame", b"c19b2d"))
        )
    }

    #[test]
    fn test_check_password_success() {
        let password = Password(16..32).fake::<String>().into_bytes();
        let client_salt = Password(8..16).fake::<String>().into_bytes();
        let client_token = to_password_token(&password, &client_salt);
        assert!(check_password(password, client_salt, &client_token).is_ok())
    }

    #[test]
    fn test_check_password_failed() {
        let password = Password(16..32).fake::<String>().into_bytes();
        let client_salt = Password(8..16).fake::<String>().into_bytes();
        let wrong_client_salt = Password(8..16).fake::<String>().into_bytes();
        let client_token = to_password_token(&password, client_salt);
        assert!(check_password(password, wrong_client_salt, &client_token).is_err())
    }
}
