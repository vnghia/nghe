use libaes::Cipher;

use super::super::OpenSubsonicError;

const IV_LEN: usize = 16;

pub fn encrypt_password(cipher: &Cipher, data: String) -> Vec<u8> {
    let plain_text = data.into_bytes();
    let iv: [u8; IV_LEN] = rand::random();
    [
        iv.as_slice(),
        cipher.cbc_encrypt(&iv, &plain_text).as_slice(),
    ]
    .concat()
}

pub fn decrypt_password(cipher: &Cipher, data: Vec<u8>) -> Result<String, OpenSubsonicError> {
    let cipher_text = &data[IV_LEN..];
    let iv = &data[..IV_LEN];
    match String::from_utf8(cipher.cbc_decrypt(iv, cipher_text)) {
        Ok(plain_text) => Ok(plain_text),
        Err(_) => Err(OpenSubsonicError::BadRequest {
            message: Some("can not decrypt password".to_owned()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_encrypt_decrypt_password() {
        let cipher = Cipher::new_128(&rand::random());

        let password: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        assert_eq!(
            password,
            decrypt_password(&cipher, encrypt_password(&cipher, password.clone())).unwrap()
        )
    }
}
