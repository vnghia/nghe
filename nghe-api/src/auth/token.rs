use nghe_proc_macro::api_derive;

#[api_derive(request = true, response = true, serde = false, eq = false)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Token(pub [u8; 16]);

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
