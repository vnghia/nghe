pub mod parsing;
mod raw;

use std::net::SocketAddr;

use derivative::Derivative;
use itertools::Itertools;
pub use parsing::ParsingConfig;

use crate::utils::fs::{LocalPath, LocalPathBuf};

#[derive(Debug)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub frontend_dir: LocalPathBuf,
}

pub type DatabaseConfig = raw::DatabaseConfig;

pub type ScanConfig = raw::ScanConfig;

#[derive(Debug, Clone)]
pub struct TranscodingConfig {
    pub buffer_size: usize,
    pub cache_dir: Option<LocalPathBuf>,
}

#[derive(Debug, Clone)]
pub struct ArtConfig {
    pub artist_dir: Option<LocalPathBuf>,
    pub song_dir: Option<LocalPathBuf>,
}

pub type LastfmConfig = raw::LastfmConfig;

pub type SpotifyConfig = raw::SpotifyConfig;

#[derive(Debug, Clone)]
pub struct ArtistIndexConfig {
    pub ignored_articles: String,
    pub ignored_prefixes: Vec<String>,
}

pub type S3Config = raw::S3Config;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Config {
    pub server: ServerConfig,
    #[derivative(Debug = "ignore")]
    pub database: DatabaseConfig,
    pub artist_index: ArtistIndexConfig,
    pub parsing: ParsingConfig,
    pub scan: ScanConfig,
    pub transcoding: TranscodingConfig,
    pub art: ArtConfig,
    #[derivative(Debug = "ignore")]
    pub lastfm: LastfmConfig,
    #[derivative(Debug = "ignore")]
    pub spotify: SpotifyConfig,
    pub s3: S3Config,
}

impl ServerConfig {
    pub fn new(raw::ServerConfig { host, port, frontend_dir }: raw::ServerConfig) -> Self {
        Self {
            bind_addr: SocketAddr::new(host, port),
            frontend_dir: LocalPath::new(&frontend_dir)
                .absolutize()
                .expect("failed to canonicalize frontend dir"),
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
        Self { ignored_articles, ignored_prefixes }
    }
}

impl TranscodingConfig {
    pub fn new(raw: raw::TranscodingConfig) -> Self {
        Self { buffer_size: raw.buffer_size, cache_dir: to_path_config(raw.cache_dir) }
    }
}

impl ArtConfig {
    pub fn new(raw: raw::ArtConfig) -> Self {
        Self { artist_dir: to_path_config(raw.artist_dir), song_dir: to_path_config(raw.song_dir) }
    }
}

impl Config {
    pub fn new() -> Self {
        let raw_config = raw::Config::default();

        let server = ServerConfig::new(raw_config.server);

        let database = raw_config.database;

        let artist_index = ArtistIndexConfig::new(raw_config.artist.ignored_articles);

        let parsing = raw_config.parsing;

        let scan = raw_config.scan;

        let transcoding = TranscodingConfig::new(raw_config.transcoding);

        let art = ArtConfig::new(raw_config.art);

        let lastfm = raw_config.lastfm;

        let spotify = raw_config.spotify;

        let s3 = raw_config.s3;

        Self {
            server,
            database,
            artist_index,
            parsing,
            scan,
            transcoding,
            art,
            lastfm,
            spotify,
            s3,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn to_path_config(p: Option<String>) -> Option<LocalPathBuf> {
    match p {
        Some(p) => {
            if LocalPath::new(&p).is_absolute() {
                Some(p.into())
            } else {
                None
            }
        }
        None => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl Default for TranscodingConfig {
        fn default() -> Self {
            Self::new(Default::default())
        }
    }

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
            vec!["The ", "A ", "An "].into_iter().map(std::borrow::ToOwned::to_owned).collect_vec()
        );
    }

    #[test]
    fn test_to_path_config() {
        assert_eq!(to_path_config(None), None);

        assert_eq!(to_path_config(Some("non-absolute".into())), None);

        let abs_path =
            std::env::temp_dir().canonicalize().unwrap().into_os_string().into_string().unwrap();
        assert_eq!(to_path_config(Some(abs_path.clone())), Some(abs_path.into()));
    }
}
