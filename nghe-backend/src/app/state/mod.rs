mod database;
mod filesystem;

pub use database::{Database, Key};
pub use filesystem::Filesystem;

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
