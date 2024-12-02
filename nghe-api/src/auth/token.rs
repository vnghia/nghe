use nghe_proc_macro::api_derive;

#[api_derive(request = false, response = false, eq = false)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "test"), derive(Default))]
pub struct Token([u8; 16]);

impl Token {
    pub fn new(password: impl AsRef<[u8]>, salt: impl AsRef<[u8]>) -> Self {
        let password = password.as_ref();
        let salt = salt.as_ref();

        let mut data = Vec::with_capacity(password.len() + salt.len());
        data.extend_from_slice(password);
        data.extend_from_slice(salt);
        Self(md5::compute(data).into())
    }
}

mod serde {
    use ::serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    impl Serialize for Token {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            faster_hex::nopfx_ignorecase::serialize(self.0, serializer)
        }
    }

    impl<'de> Deserialize<'de> for Token {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let data: Vec<u8> = faster_hex::nopfx_ignorecase::deserialize(deserializer)?;
            Ok(Token(data.try_into().map_err(|_| {
                de::Error::custom("Could not convert vector to array of length 16")
            })?))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{from_value, json};

    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(
            from_value::<Token>(json!("26719a1196d2a940705a59634eb18eab")).unwrap(),
            Token::new(b"sesame", "c19b2d")
        );
    }
}
