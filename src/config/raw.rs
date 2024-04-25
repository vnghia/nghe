use std::net::IpAddr;
use std::path::PathBuf;

use derivative::Derivative;
use figment::providers::{Env, Serialized};
use figment::Figment;
use nghe_types::constant::SERVER_NAME;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull};

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
    #[derivative(Default(
        value = "std::env::current_dir().unwrap().join(\"frontend\").join(\"dist\")"
    ))]
    pub frontend_dir: PathBuf,
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

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct ArtistConfig {
    #[derivative(Default(value = "\"The An A Die Das Ein Eine Les Le La\".into()"))]
    pub ignored_articles: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct ScanConfig {
    #[derivative(Default(value = "false"))]
    pub parallel: bool,
    #[derivative(Default(value = "10"))]
    pub channel_size: usize,
    #[derivative(Default(value = "10"))]
    pub pool_size: usize,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct TranscodingConfig {
    #[derivative(Default(value = "32 * 1024"))]
    pub buffer_size: usize,
    #[derivative(Default(
        value = "Some(std::env::temp_dir().join(\"nghe\").join(\"cache\").join(\"transcoding\"))"
    ))]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub cache_path: Option<PathBuf>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Derivative)]
#[derivative(Default)]
pub struct ArtConfig {
    #[derivative(Default(
        value = "Some(std::env::temp_dir().join(\"nghe\").join(\"art\").join(\"artist\"))"
    ))]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub artist_dir: Option<PathBuf>,
    #[derivative(Default(
        value = "Some(std::env::temp_dir().join(\"nghe\").join(\"art\").join(\"song\"))"
    ))]
    #[serde_as(deserialize_as = "DefaultOnNull")]
    pub song_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastfmConfig {
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpotifyConfig {
    pub id: Option<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub artist: ArtistConfig,
    pub parsing: ParsingConfig,
    pub scan: ScanConfig,
    pub transcoding: TranscodingConfig,
    pub art: ArtConfig,
    pub lastfm: LastfmConfig,
    pub spotify: SpotifyConfig,
}

impl Config {
    pub fn new() -> Self {
        Figment::new()
            .merge(Env::prefixed(constcat::concat!(SERVER_NAME, "_")).split("__"))
            .join(Serialized::default("server", ServerConfig::default()))
            .join(Serialized::default("artist", ArtistConfig::default()))
            .join(Serialized::default("parsing", ParsingConfig::default()))
            .join(Serialized::default("scan", ScanConfig::default()))
            .join(Serialized::default("transcoding", TranscodingConfig::default()))
            .join(Serialized::default("art", ArtConfig::default()))
            .join(Serialized::default("lastfm", LastfmConfig::default()))
            .join(Serialized::default("spotify", SpotifyConfig::default()))
            .extract()
            .expect("can not parse initial config")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
