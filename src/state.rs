use sea_orm::{Database, DatabaseConnection};

use crate::config::{Config, EncryptionKey};

#[derive(Debug, Default, Clone)]
pub struct ServerState {
    pub conn: DatabaseConnection,
    pub encryption_key: EncryptionKey,
}

impl ServerState {
    pub async fn new(config: &Config) -> Self {
        // database
        let conn = Database::connect(&config.database.url)
            .await
            .expect("can not connect to the database");

        Self {
            conn,
            encryption_key: config.database.encryption_key,
        }
    }
}
