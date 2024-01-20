use crate::config::{Config, EncryptionKey};
use crate::DbPool;

use diesel_async::pooled_connection::AsyncDieselConnectionManager;

#[derive(Clone)]
pub struct ServerState {
    pub pool: DbPool,
    pub encryption_key: EncryptionKey,
}

impl ServerState {
    pub async fn new(config: &Config) -> Self {
        // database
        let pool = DbPool::builder(AsyncDieselConnectionManager::<
            diesel_async::AsyncPgConnection,
        >::new(&config.database.url))
        .build()
        .expect("can not connect to the database");

        Self {
            pool,
            encryption_key: config.database.encryption_key,
        }
    }
}
