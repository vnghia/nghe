use nghe_proc_macro::api_derive;
use serde_with::serde_as;

#[serde_as]
#[api_derive(response = false)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "test", derive(serde::Serialize))]
pub struct Token(#[serde_as(as = "serde_with::hex::Hex")] [u8; 16]);

#[api_derive(response = false, fake = false)]
#[derive(Clone, Copy)]
#[cfg_attr(feature = "test", derive(serde::Serialize))]
pub struct Auth<'u, 't> {
    #[serde(rename = "u")]
    pub username: &'u str,
    #[serde(rename = "s")]
    pub salt: &'t str,
    #[serde(rename = "t")]
    pub token: Token,
}

#[api_derive(serde = false, response = false, fake = false)]
pub struct BinaryRequest<'u, 't, R> {
    pub auth: Auth<'u, 't>,
    pub request: R,
}

impl Auth<'_, '_> {
    pub fn tokenize(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> Token {
        let password = password.as_ref();
        let salt = salt.as_ref();

        let mut data = Vec::with_capacity(password.len() + salt.len());
        data.extend_from_slice(password);
        data.extend_from_slice(salt);
        Token(md5::compute(data).into())
    }

    pub fn check(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>, token: &Token) -> bool {
        let password = password.as_ref();
        let salt = salt.as_ref();

        let password_token = Self::tokenize(password, salt);
        &password_token == token
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::Password;
    use fake::Fake;
    use serde_json::{from_value, json};

    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(
            from_value::<Token>(json!("26719a1196d2a940705a59634eb18eab")).unwrap(),
            Auth::tokenize(b"sesame", b"c19b2d")
        );
    }

    #[test]
    fn test_check_success() {
        let password = Password(16..32).fake::<String>().into_bytes();
        let client_salt = Password(8..16).fake::<String>().into_bytes();
        let client_token = Auth::tokenize(&password, &client_salt);
        assert!(Auth::check(password, client_salt, &client_token));
    }

    #[test]
    fn test_check_failed() {
        let password = Password(16..32).fake::<String>().into_bytes();
        let client_salt = Password(8..16).fake::<String>().into_bytes();
        let wrong_client_salt = Password(8..16).fake::<String>().into_bytes();
        let client_token = Auth::tokenize(&password, client_salt);
        assert!(!Auth::check(password, wrong_client_salt, &client_token));
    }
}
