mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use derivative::Derivative;
use libaes::AES_128_KEY_LEN;
use serde::Deserialize;
use serde_with::serde_as;
use std::{net::IpAddr, path::PathBuf};

pub type EncryptionKey = [u8; AES_128_KEY_LEN];

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Server {
    pub host: IpAddr,
    pub port: u16,
}

#[serde_as]
#[derive(Derivative, Deserialize, Clone)]
#[derivative(Debug)]
#[allow(unused)]
pub struct Database {
    #[derivative(Debug = "ignore")]
    pub url: String,
    #[derivative(Debug = "ignore")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub encryption_key: EncryptionKey,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Folder {
    pub top_paths: Vec<PathBuf>,
    pub depth_levels: Vec<u8>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub folder: Folder,
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        let s = config::Config::builder()
            // server
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .set_default("folder.depth_levels", Vec::<u8>::default())?
            .add_source(
                config::Environment::with_prefix(built_info::PKG_NAME)
                    .prefix_separator("_")
                    .separator("__")
                    .list_separator(":")
                    .try_parsing(true)
                    .with_list_parse_key("folder.top_paths")
                    .with_list_parse_key("folder.depth_levels"),
            )
            .build()?;
        s.try_deserialize()
    }
}
