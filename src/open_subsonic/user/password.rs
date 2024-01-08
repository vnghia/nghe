use libaes::Cipher;

use super::super::{OSResult, OpenSubsonicError};

const IV_LEN: usize = 16;

pub fn encrypt_password(cipher: &Cipher, data: &String) -> Vec<u8> {
    let plain_text = data.as_bytes();
    let iv: [u8; IV_LEN] = rand::random();
    [
        iv.as_slice(),
        cipher.cbc_encrypt(&iv, plain_text).as_slice(),
    ]
    .concat()
}

pub fn decrypt_password(cipher: &Cipher, data: &Vec<u8>) -> OSResult<String> {
    let cipher_text = &data[IV_LEN..];
    let iv = &data[..IV_LEN];
    match String::from_utf8(cipher.cbc_decrypt(iv, cipher_text)) {
        Ok(plain_text) => Ok(plain_text),
        Err(_) => Err(OpenSubsonicError::BadRequest {
            message: Some("can not decrypt password".to_owned()),
        }),
    }
}

fn to_hex_string(digest: md5::Digest) -> String {
    format!("{:x}", digest)
}

pub fn check_password(
    password: String,
    client_salt: &String,
    client_token: &String,
) -> OSResult<()> {
    let password_token = to_hex_string(md5::compute(password + client_salt));
    if &password_token == client_token {
        Ok(())
    } else {
        Err(OpenSubsonicError::Unauthorized { message: None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    fn generate_alphanumeric_string(length: usize) -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }

    #[test]
    fn test_roundtrip_password() {
        let cipher = Cipher::new_128(&rand::random());
        let password: String = generate_alphanumeric_string(32);
        assert_eq!(
            password,
            decrypt_password(&cipher, &encrypt_password(&cipher, &password)).unwrap()
        )
    }

    #[test]
    fn test_check_password_success() {
        let password: String = generate_alphanumeric_string(32);
        let client_salt: String = generate_alphanumeric_string(8);
        let client_token = to_hex_string(md5::compute(password.clone() + &client_salt));
        assert!(check_password(password, &client_salt, &client_token).is_ok())
    }

    #[test]
    fn test_check_password_failed() {
        let password: String = generate_alphanumeric_string(32);
        let client_salt: String = generate_alphanumeric_string(8);
        let wrong_client_salt = generate_alphanumeric_string(8);
        let client_token = to_hex_string(md5::compute(password.clone() + &client_salt));
        assert!(check_password(password, &wrong_client_salt, &client_token).is_err())
    }
}
