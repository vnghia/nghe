mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use derivative::Derivative;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Derivative, Debug, Deserialize, Clone)]
#[derivative(Default)]
#[allow(unused)]
pub struct Server {
    #[derivative(Default(value = "\"127.0.0.1\".parse::<IpAddr>().unwrap()"))]
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Derivative, Default, Deserialize, Clone)]
#[derivative(Debug)]
#[allow(unused)]
pub struct Database {
    #[derivative(Debug = "ignore")]
    pub url: String,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[allow(unused)]
pub struct Config {
    pub server: Server,
    pub database: Database,
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        let s = config::Config::builder()
            // server
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .add_source(
                config::Environment::with_prefix(built_info::PKG_NAME)
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;
        s.try_deserialize()
    }
}
