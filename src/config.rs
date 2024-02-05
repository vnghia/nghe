mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use derivative::Derivative;
use figment::{
    providers::{Env, Serialized},
    util::map,
    Figment,
};
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
    pub key: EncryptionKey,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Folder {
    pub top_paths: Vec<PathBuf>,
    pub depth_levels: Vec<u8>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Artist {
    pub ignored_articles: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub folder: Folder,
    pub artist: Artist,
}

impl Config {
    pub fn new() -> Self {
        Figment::new()
            .merge(
                Env::prefixed(&concat_string::concat_string!(built_info::PKG_NAME, "_"))
                    .split("__"),
            )
            .join(Serialized::default(
                "server",
                map!["host" => "127.0.0.1", "port" => "3000"],
            ))
            .join(Serialized::default(
                "folder.depth_levels",
                Vec::<u8>::default(),
            ))
            .join(Serialized::default(
                "artist",
                map!["ignored_articles" => "The An A Die Das Ein Eine Les Le La"],
            ))
            .extract()
            .expect("can not parse initial config")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
