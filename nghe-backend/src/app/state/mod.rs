mod database;

pub use database::{Database, Key};

use crate::config;

#[derive(Clone)]
pub struct App {
    pub database: Database,
}

impl App {
    pub fn new(database: &config::Database) -> Self {
        Self { database: Database::new(database) }
    }
}
