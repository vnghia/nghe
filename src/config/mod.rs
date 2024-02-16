mod raw;

use derivative::Derivative;
use itertools::Itertools;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
}

pub type DatabaseConfig = raw::DatabaseConfig;

pub type FolderConfig = raw::FolderConfig;

#[derive(Debug, Clone, Default)]
pub struct ArtistConfig {
    pub ignored_articles: String,
    pub ignored_prefixes: Vec<String>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Config {
    pub server: ServerConfig,
    #[derivative(Debug = "ignore")]
    pub database: DatabaseConfig,
    pub folder: FolderConfig,
    pub artist: ArtistConfig,
}

impl ServerConfig {
    pub fn new(host: IpAddr, port: u16) -> Self {
        Self {
            bind_addr: SocketAddr::new(host, port),
        }
    }
}

impl ArtistConfig {
    pub fn new(ignored_articles: String) -> Self {
        let ignored_prefixes = ignored_articles
            .split_ascii_whitespace()
            .map(|v| concat_string::concat_string!(v, " "))
            .collect_vec();
        Self {
            ignored_articles,
            ignored_prefixes,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let raw_config = raw::Config::default();

        let server = ServerConfig::new(raw_config.server.host, raw_config.server.port);

        let database = raw_config.database;

        let folder = raw_config.folder;

        let artist = ArtistConfig::new(raw_config.artist.ignored_articles);

        Self {
            server,
            database,
            folder,
            artist,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_artist_config() {
        let ignored_articles = "The A An".to_owned();
        let artist_config = ArtistConfig::new(ignored_articles);
        assert_eq!(
            artist_config.ignored_prefixes,
            vec!["The ", "A ", "An "]
                .into_iter()
                .map(std::borrow::ToOwned::to_owned)
                .collect_vec()
        );
    }
}
