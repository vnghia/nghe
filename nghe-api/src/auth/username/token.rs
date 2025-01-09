use std::borrow::Cow;

#[cfg(feature = "fake")]
use fake::{Fake, Faker};
use nghe_proc_macro::api_derive;

#[api_derive(request = false, response = false, fake = true)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Token([u8; 16]);

#[api_derive(fake = true)]
#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Auth<'s> {
    #[serde(rename = "s")]
    #[cfg_attr(feature = "fake", dummy(expr = "Faker.fake::<String>().into()"))]
    pub salt: Cow<'s, str>,
    #[serde(rename = "t")]
    pub token: Token,
}

mod serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

    use super::Token;

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

#[cfg(any(test, feature = "backend"))]
mod backend {
    use super::*;

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
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[api_derive]
    #[cfg_attr(test, derive(PartialEq))]
    pub struct Test {
        token: Token,
    }

    #[rstest]
    #[case("token=26719a1196d2a940705a59634eb18eab", Some(Token::new(b"sesame", "c19b2d")))]
    #[case("token=26719a1196d2a940705a59634eb18eab1", None)]
    fn test_deserialize(#[case] input: &str, #[case] result: Option<Token>) {
        assert_eq!(serde_html_form::from_str(input).ok(), result.map(|token| Test { token }));
    }
}
