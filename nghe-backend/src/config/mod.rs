mod database;
pub mod filesystem;
mod index;
pub mod parsing;
mod server;
mod transcode;

pub use database::Database;
use figment::providers::{Env, Serialized};
use figment::Figment;
use filesystem::Filesystem;
pub use index::Index;
use nghe_api::constant;
pub use parsing::Parsing;
use serde::Deserialize;
pub use server::Server;
pub use transcode::Transcode;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
    pub database: Database,
    pub filesystem: Filesystem,
    pub parsing: Parsing,
    pub index: Index,
    pub transcode: Transcode,
}

impl Default for Config {
    fn default() -> Self {
        Figment::new()
            .merge(Env::prefixed(const_format::concatc!(constant::SERVER_NAME, "_")).split("__"))
            .join(Serialized::default("server", Server::default()))
            .join(Serialized::default("filesystem", Filesystem::default()))
            .join(Serialized::default("parsing", Parsing::default()))
            .join(Serialized::default("index", Index::default()))
            .join(Serialized::default("transcode", Transcode::default()))
            .extract()
            .expect("Could not parse config")
    }
}
