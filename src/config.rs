mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use derivative::Derivative;
use libaes::AES_128_KEY_LEN;
use serde::{de::Error, Deserialize, Deserializer};
use std::net::IpAddr;

pub type EncryptionKey = [u8; AES_128_KEY_LEN];

#[derive(Derivative, Debug, Deserialize, Clone)]
#[derivative(Default)]
#[allow(unused)]
pub struct Server {
    #[derivative(Default(value = "\"127.0.0.1\".parse::<IpAddr>().unwrap()"))]
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Derivative, Default, Deserialize, Clone)]
#[derivative(Debug)]
#[allow(unused)]
pub struct Database {
    #[derivative(Debug = "ignore")]
    pub url: String,
    #[derivative(Debug = "ignore")]
    #[serde(deserialize_with = "string_to_key")]
    pub encryption_key: EncryptionKey,
}

fn string_to_key<'de, D>(deserializer: D) -> Result<EncryptionKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.len() != AES_128_KEY_LEN {
        Err(D::Error::custom(
            "encryption key length should be 128-bit or 16 ascii character",
        ))
    } else {
        s.as_bytes().try_into().map_err(D::Error::custom)
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub server: Server,
    pub database: Database,
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        let s = config::Config::builder()
            // server
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .add_source(
                config::Environment::with_prefix(built_info::PKG_NAME)
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;
        s.try_deserialize()
    }
}
