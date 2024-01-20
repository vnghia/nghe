use crate::config::{Config, EncryptionKey};
use crate::DatabasePool;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;

#[derive(Clone)]
pub struct DatabaseState {
    pub pool: DatabasePool,
    pub key: EncryptionKey,
}

#[derive(Clone)]
pub struct ServerState {
    pub database: DatabaseState,
}

impl ServerState {
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
        }
    }
}
