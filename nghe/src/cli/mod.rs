mod error;
mod rest;

use conf::{Conf, Subcommands};
use error::Error;
use rest::Rest;

use crate::server;

#[derive(Debug, Subcommands)]
enum Action {
    Rest(Rest),
}

#[derive(Debug, Default, Subcommands)]
enum Entrypoint {
    #[default]
    Server,
    Cli {
        #[conf(subcommands)]
        action: Action,
    },
}

#[derive(Debug, Conf)]
#[conf(version_fn = version)]
struct Cli {
    #[conf(subcommands)]
    entrypoint: Option<Entrypoint>,
}

fn version() -> &'static str {
    nghe_api::constant::SERVER_VERSION
}

pub async fn entrypoint() {
    let cli = Cli::parse();
    let entrypoint = cli.entrypoint.unwrap_or_default();
    match entrypoint {
        Entrypoint::Server => server::start().await,
        Entrypoint::Cli { action } => match action {
            Action::Rest(rest) => rest.run().await.unwrap(),
        },
    }
}
