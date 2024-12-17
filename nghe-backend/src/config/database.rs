use educe::Educe;
use serde::{Deserialize, Deserializer, de};
use serde_with::serde_as;

use crate::database::Key;

#[serde_as]
#[derive(Deserialize, Educe)]
#[educe(Debug)]
pub struct Database {
    #[educe(Debug(ignore))]
    pub url: String,
    #[serde(deserialize_with = "deserialize")]
    #[educe(Debug(ignore))]
    pub key: Key,
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: Deserializer<'de>,
{
    let data: Vec<u8> = faster_hex::nopfx_ignorecase::deserialize(deserializer)?;
    data.try_into().map_err(|_| de::Error::custom("Could not convert vector to array of length 16"))
}
