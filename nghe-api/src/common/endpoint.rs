use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct AuthToken(#[serde_as(as = "serde_with::hex::Hex")] [u8; 16]);

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Auth<'u, 's> {
    #[serde(rename = "u")]
    pub username: &'u str,
    #[serde(rename = "s")]
    pub salt: &'s str,
    #[serde(rename = "t")]
    pub token: AuthToken,
}

pub trait Endpoint {
    type Response: Serialize + DeserializeOwned;
}
