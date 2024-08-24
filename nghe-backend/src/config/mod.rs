mod server;

use figment::providers::{Env, Serialized};
use figment::Figment;
use nghe_api::constant;
use serde::Deserialize;
pub use server::Server;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: Server,
}

impl Default for Config {
    fn default() -> Self {
        Figment::new()
            .merge(Env::prefixed(const_format::concatc!(constant::SERVER_NAME, "_")).split("__"))
            .join(Serialized::default("server", Server::default()))
            .extract()
            .expect("Could not parse config")
    }
}
