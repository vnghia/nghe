mod raw;

use derivative::Derivative;
use itertools::Itertools;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
}

pub type DatabaseConfig = raw::DatabaseConfig;

pub type FolderConfig = raw::FolderConfig;

#[derive(Debug, Clone)]
pub struct ArtistIndexConfig {
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
    pub artist_index: ArtistIndexConfig,
}

impl ServerConfig {
    pub fn new(raw::ServerConfig { host, port }: raw::ServerConfig) -> Self {
        Self {
            bind_addr: SocketAddr::new(host, port),
        }
    }
}

impl ArtistIndexConfig {
    pub const IGNORED_ARTICLES_CONFIG_KEY: &'static str = "ignored_articles";

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

        let server = ServerConfig::new(raw_config.server);

        let database = raw_config.database;

        let folder = raw_config.folder;

        let artist_index = ArtistIndexConfig::new(raw_config.artist.ignored_articles);

        Self {
            server,
            database,
            folder,
            artist_index,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Default for ArtistIndexConfig {
        fn default() -> Self {
            Self::new(raw::ArtistConfig::default().ignored_articles)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_artist_config() {
        let ignored_articles = "The A An".to_owned();
        let artist_index_config = ArtistIndexConfig::new(ignored_articles);
        assert_eq!(
            artist_index_config.ignored_prefixes,
            vec!["The ", "A ", "An "]
                .into_iter()
                .map(std::borrow::ToOwned::to_owned)
                .collect_vec()
        );
    }
}
