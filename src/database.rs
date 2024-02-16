use crate::{migration, DatabasePool};

use diesel_async::pooled_connection::AsyncDieselConnectionManager;

pub type EncryptionKey = [u8; libaes::AES_128_KEY_LEN];

#[derive(Clone)]
pub struct Database {
    pub pool: DatabasePool,
    pub key: EncryptionKey,
}

impl Database {
    pub async fn new(url: &str, key: EncryptionKey) -> Self {
        let pool = DatabasePool::builder(AsyncDieselConnectionManager::<
            diesel_async::AsyncPgConnection,
        >::new(url))
        .build()
        .expect("can not connect to the database");
        migration::run_pending_migrations(url).await;
        Self { pool, key }
    }
}
