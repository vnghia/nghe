mod error;
mod rest;

use conf::{Conf, Subcommands};
use error::Error;
use rest::Rest;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use crate::server;

#[derive(Debug, Conf)]
struct Log {
    #[conf(long, env)]
    config: Option<String>,
}

#[derive(Debug, Subcommands)]
enum Action {
    Rest(Rest),
}

#[derive(Debug, Conf)]
#[conf(env_prefix = "NGHE_CLI_")]
struct Cli {
    #[conf(flatten, prefix)]
    log: Log,
    #[conf(subcommands)]
    action: Action,
}

#[derive(Debug, Default, Subcommands)]
enum Entrypoint {
    #[default]
    Server,
    Cli(Cli),
}

#[derive(Debug, Conf)]
#[conf(version_fn = Self::version)]
struct Main {
    #[conf(subcommands)]
    entrypoint: Option<Entrypoint>,
}

impl Main {
    fn version() -> &'static str {
        nghe_api::constant::SERVER_VERSION
    }
}

fn init_tracing(log: &Log) -> Result<(), Error> {
    server::init_eyre()?;

    let tracing = tracing_subscriber::registry()
        .with(log.config.as_ref().map_or_else(
            || {
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| if cfg!(test) { "debug" } else { "info" }.into())
            },
            Into::into,
        ))
        .with(tracing_error::ErrorLayer::default());

    let tracing_layer = tracing_subscriber::fmt::layer();

    tracing.with(tracing_layer.with_writer(std::io::stderr)).try_init().map_err(Into::into)
}

pub async fn entrypoint() {
    let main = Main::parse();
    let entrypoint = main.entrypoint.unwrap_or_default();
    match entrypoint {
        Entrypoint::Server => server::start().await,
        Entrypoint::Cli(cli) => {
            init_tracing(&cli.log).unwrap();

            tracing::info!(?cli);
            match cli.action {
                Action::Rest(rest) => rest.run().await.unwrap(),
            }
        }
    }
}
