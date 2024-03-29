mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use std::net::IpAddr;
use std::path::PathBuf;

use derivative::Derivative;
use figment::providers::{Env, Serialized};
use figment::Figment;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::parsing::ParsingConfig;
use crate::database::EncryptionKey;

#[derive(Debug, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
#[serde(default)]
pub struct ServerConfig {
    #[derivative(Default(value = "[127u8, 0u8, 0u8, 1u8].into()"))]
    pub host: IpAddr,
    #[derivative(Default(value = "3000"))]
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

#[derive(Debug, Deserialize)]
pub struct FolderConfig {
    pub top_paths: Vec<PathBuf>,
    pub depth_levels: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct ArtistConfig {
    #[derivative(Default(value = "\"The An A Die Das Ein Eine Les Le La\".into()"))]
    pub ignored_articles: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct ScanConfig {
    #[derivative(Default(value = "100"))]
    pub channel_size: usize,
    #[derivative(Default(value = "10"))]
    pub scan_media_task_size: usize,
    #[derivative(Default(value = "100"))]
    pub process_path_task_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct TranscodingConfig {
    #[derivative(Default(value = "32 * 1024"))]
    pub buffer_size: usize,
    #[derivative(Default(
        value = "Some(std::env::temp_dir().join(\"nghe\").join(\"cache\").join(\"transcoding\"))"
    ))]
    pub cache_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub folder: FolderConfig,
    pub artist: ArtistConfig,
    pub parsing: ParsingConfig,
    pub scan: ScanConfig,
    pub transcoding: TranscodingConfig,
}

impl Config {
    pub fn new() -> Self {
        Figment::new()
            .merge(Env::prefixed(constcat::concat!(built_info::PKG_NAME, "_")).split("__"))
            .join(Serialized::default("server", ServerConfig::default()))
            .join(Serialized::default("folder.depth_levels", Vec::<usize>::default()))
            .join(Serialized::default("artist", ArtistConfig::default()))
            .join(Serialized::default("parsing", ParsingConfig::default()))
            .join(Serialized::default("scan", ScanConfig::default()))
            .join(Serialized::default("transcoding", TranscodingConfig::default()))
            .extract()
            .expect("can not parse initial config")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
