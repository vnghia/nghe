use super::database;
use crate::app::state;

pub struct Mock {
    pub database: database::Mock,
}

impl Mock {
    pub async fn new() -> Self {
        let database = database::Mock::new().await;

        Self { database }
    }

    pub fn database(&self) -> &state::Database {
        &self.database.state
    }
}
