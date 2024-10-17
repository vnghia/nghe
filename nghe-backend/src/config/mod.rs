mod database;
pub mod filesystem;
mod index;
pub mod parsing;
mod server;

pub use database::Database;
use figment::providers::{Env, Serialized};
use figment::Figment;
use filesystem::Filesystem;
pub use index::Index;
use nghe_api::constant;
pub use parsing::Parsing;
use serde::Deserialize;
pub use server::Server;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub filesystem: Filesystem,
    pub parsing: Parsing,
    pub index: Index,
}

impl Default for Config {
    fn default() -> Self {
        Figment::new()
            .merge(Env::prefixed(const_format::concatc!(constant::SERVER_NAME, "_")).split("__"))
            .join(Serialized::default("server", Server::default()))
            .join(Serialized::default("filesystem", Filesystem::default()))
            .join(Serialized::default("parsing", Parsing::default()))
            .extract()
            .expect("Could not parse config")
    }
}
