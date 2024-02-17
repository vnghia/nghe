mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use crate::database::EncryptionKey;

use derivative::Derivative;
use figment::{
    providers::{Env, Serialized},
    Figment,
};
use serde::Deserialize;
use serde_with::serde_as;
use std::{net::IpAddr, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

#[serde_as]
#[derive(Derivative, Deserialize)]
#[derivative(Debug)]
pub struct DatabaseConfig {
    #[derivative(Debug = "ignore")]
    pub url: String,
    #[derivative(Debug = "ignore")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub key: EncryptionKey,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct FolderConfig {
    pub top_paths: Vec<PathBuf>,
    pub depth_levels: Vec<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistConfig {
    pub ignored_articles: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub folder: FolderConfig,
    pub artist: ArtistConfig,
}

impl ArtistConfig {
    pub const IGNORED_ARTICLES_DEFAULT_VALUE: &'static str = "The An A Die Das Ein Eine Les Le La";
}

impl Config {
    pub fn new() -> Self {
        Figment::new()
            .merge(
                Env::prefixed(&concat_string::concat_string!(built_info::PKG_NAME, "_"))
                    .split("__"),
            )
            .join(Serialized::default("server.host", "127.0.0.1"))
            .join(Serialized::default("server.port", 3000))
            .join(Serialized::default(
                "folder.depth_levels",
                Vec::<usize>::default(),
            ))
            .join(Serialized::default(
                "artist.ignored_articles",
                ArtistConfig::IGNORED_ARTICLES_DEFAULT_VALUE,
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
