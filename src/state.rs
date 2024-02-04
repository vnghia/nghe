use crate::config::{Config, EncryptionKey};
use crate::DatabasePool;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use itertools::Itertools;

#[derive(Clone)]
pub struct DatabaseState {
    pub pool: DatabasePool,
    pub key: EncryptionKey,
}

#[derive(Clone, Default)]
pub struct ArtistState {
    pub ignored_articles: String,
    pub ignored_prefixes: Vec<String>,
}

#[derive(Clone)]
pub struct ServerState {
    pub database: DatabaseState,
    pub artist: ArtistState,
}

impl ServerState {
    pub fn build_artist_state(ignored_articles: &str) -> ArtistState {
        ArtistState {
            ignored_articles: ignored_articles.to_owned(),
            ignored_prefixes: ignored_articles
                .split_ascii_whitespace()
                .map(|v| concat_string::concat_string!(v, " "))
                .collect_vec(),
        }
    }

    pub async fn new(config: &Config) -> Self {
        // database
        let pool = DatabasePool::builder(AsyncDieselConnectionManager::<
            diesel_async::AsyncPgConnection,
        >::new(&config.database.url))
        .build()
        .expect("can not connect to the database");

        Self {
            database: DatabaseState {
                pool,
                key: config.database.key,
            },
            artist: Self::build_artist_state(&config.artist.ignored_articles),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::Itertools;

    #[test]
    fn test_build_artists_state() {
        let ignored_articles = "The A An";
        let artists_state = ServerState::build_artist_state(ignored_articles);
        assert_eq!(artists_state.ignored_articles, ignored_articles);
        assert_eq!(
            artists_state.ignored_prefixes,
            vec!["The ", "A ", "An "]
                .into_iter()
                .map(std::borrow::ToOwned::to_owned)
                .collect_vec()
        );
    }
}
