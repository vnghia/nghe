use sea_orm::{Database, DatabaseConnection};

use crate::config::Config;

#[derive(Debug, Default, Clone)]
pub struct ServerState {
    pub config: Config,
    pub conn: DatabaseConnection,
}

impl ServerState {
    pub async fn new(config: Config) -> Self {
        // database
        let conn = Database::connect(&config.database.url)
            .await
            .expect("can not connect to the database");

        Self { config, conn }
    }
}
